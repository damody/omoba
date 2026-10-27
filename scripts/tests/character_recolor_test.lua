-- Contract cases for scripts/character_recolor.lua: rule validation, embedded-texture extraction from a
-- synthetic binary FBX, plan preparation through a fake host, and commandlet result verification.
-- Pixel semantics (layout, alpha and untouched pixels) are covered by tools/lua-host png_recolor tests.
local script=debug.getinfo(1,'S').source:sub(2);package.path=assert(script:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,h=b.lib('path'),b.lib('hash');local m=require('character_recolor')
local total=0
local function check(name,ok)assert(ok,name);total=total+1 end
local function fails(name,f)check(name,not pcall(f))end

local rule={hue={330,15},saturation={0,0.45},value={0.5,1},target_hue=345,target_saturation=0.5,value_scale=0.92}
local function copy(t)local o={};for k,v in pairs(t)do o[k]=type(v)=='table'and copy(v)or v end;return o end
check('absent recolor is allowed',m.validate_recolor('oichi',nil)==nil)
check('valid recolor passes',m.validate_recolor('oichi',{variant='sakura',rules={rule}}).variant=='sakura')
for name,change in pairs({
  bad_variant=function(r)r.variant='Sakura!'end, no_rules=function(r)r.rules={}end,
  too_many_rules=function(r)for i=1,17 do r.rules[i]=copy(rule)end end,
  hue_out_of_range=function(r)r.rules[1].hue={330,400}end, descending_saturation=function(r)r.rules[1].saturation={0.5,0.1}end,
  saturation_target_out_of_range=function(r)r.rules[1].target_saturation=1.5 end, value_scale_zero=function(r)r.rules[1].value_scale=0 end,
  nan_target=function(r)r.rules[1].target_hue=0/0 end,
})do
  local recolor={variant='sakura',rules={copy(rule)}};change(recolor)
  fails('rejects '..name,function()m.validate_recolor('oichi',recolor)end)
end

-- Minimal binary FBX 7400: Objects > Video > {RelativeFilename "S", Content "R"}.
local function node(name,props,children)
  local body=props..(children or'')..(children and string.rep('\0',13)or'')
  return function(offset)
    local header=13+#name;local finish=offset+header+#body
    return string.pack('<I4I4I4B',finish,#props>0 and 1 or 0,#props,#name)..name..body
  end
end
local function fbx(png)
  local function str(t,s)return t..string.pack('<I4',#s)..s end
  local head='Kaydara FBX Binary  \0\26\0'..string.pack('<I4',7400)
  -- Offsets are absolute (0-based); build innermost-first with known positions.
  local base=#head
  local rel_props=str('S','textures\\hero_basecolor.PNG');local content_props=str('R',png)
  local video_header=13+#'Video'
  local rel=node('RelativeFilename',rel_props)(base+video_header)
  local content=node('Content',content_props)(base+video_header+#rel)
  local children=rel..content
  local video=string.pack('<I4I4I4B',base+video_header+#children+13,0,0,#'Video')..'Video'..children..string.rep('\0',13)
  return head..video..string.rep('\0',13)
end
local png='\137PNG\r\n\26\n'..string.rep('x',32)
local textures=m.embedded_textures(fbx(png))
check('extracts the embedded texture',#textures==1 and textures[1].bytes==png and textures[1].name=='hero_basecolor.PNG')
check('picks the PNG base colour',m.basecolor_png(fbx(png)).bytes==png)
fails('rejects an FBX without PNG',function()m.basecolor_png(fbx('JPEG'..string.rep('y',8)))end)
fails('rejects non-FBX bytes',function()m.embedded_textures('not an fbx')end)

-- prepare: hash-pinned FBX, host call shape, bounded report.
local dir=p.join(b.root,'target','character-recolor-test');p.mkdir_p(dir)
local fbx_file=p.join(dir,'hero.fbx');p.write(fbx_file,fbx(png),true,true)
local job={id='hero',variant='sakura',rules={rule},fbx=fbx_file,fbx_sha256=h.sha256(fbx_file),source_material='/Game/Omoba/Characters/TripoV1/hero/mat.mat',
  base_png=p.join(dir,'hero_basecolor.png'),variant_png=p.join(dir,'hero_basecolor_sakura.png'),destination=m.destination..'/hero',
  texture_name='hero_basecolor_sakura',material_name='hero_mat_sakura'}
local seen
local host={call=function(op,params)seen={op=op,params=params};return{width=2,height=2,pixels=4,changed=3,output_sha256=string.rep('a',64)}end}
local row=m.prepare(job,host)
check('prepare extracts the PNG before recolouring',p.read(job.base_png,true)==png and seen.op=='png_recolor'
  and seen.params.source==job.base_png and seen.params.output==job.variant_png and seen.params.rules==job.rules)
check('prepare reports the variant row',row.texture_sha256==string.rep('a',64)and row.changed_pixels==3 and row.width==2)
fails('prepare rejects a changed FBX',function()local bad=copy(job);bad.fbx_sha256=string.rep('0',64);m.prepare(bad,host)end)
fails('prepare rejects a no-op recolour',function()m.prepare(job,{call=function()return{width=2,height=2,pixels=4,changed=0,output_sha256=''}end})end)
fails('prepare rejects a whole-image recolour',function()m.prepare(job,{call=function()return{width=2,height=2,pixels=4,changed=4,output_sha256=''}end})end)

-- verify_result
local plan={schema=m.schema,variants={row}}
local function result(change)
  local r={schema='omoba.character-recolor-result.v1',success=true,error='',imported=1,verified=0,variants={{id='hero',variant='sakura',action='imported',
    texture=row.destination..'/hero_basecolor_sakura.hero_basecolor_sakura',material=m.material_path(row),texture_width=2,texture_height=2,replaced_samples=1}}}
  if change then change(r)end;return r
end
check('accepts an imported variant',m.verify_result(plan,result()))
check('accepts a verified repeat',m.verify_result(plan,result(function(r)r.imported=0;r.verified=1;r.variants[1].action='verified'end)))
for name,change in pairs({
  failure=function(r)r.success=false end, error_text=function(r)r.error='x'end, wrong_schema=function(r)r.schema='old'end,
  missing_variant=function(r)r.variants={}end, unknown_variant=function(r)r.variants[1].id='other'end, other_variant=function(r)r.variants[1].variant='ally'end,
  wrong_texture=function(r)r.variants[1].texture='/Game/Elsewhere/t.t'end, wrong_material=function(r)r.variants[1].material='/Game/Elsewhere/m.m'end,
  resized_texture=function(r)r.variants[1].texture_width=1 end, unsampled=function(r)r.variants[1].replaced_samples=0 end,
  bad_action=function(r)r.variants[1].action='skipped'end, wrong_counters=function(r)r.imported=0 end,
})do fails('rejects '..name,function()m.verify_result(plan,result(change))end)end
print('PASS '..total..' character recolor contract cases; no Unreal import')
