-- Build-time colour variants of generated (Tripo) hero bodies. A hero manifest may carry
--   recolor={variant='sakura',rules={{hue={330,15},saturation={0,0.4},value={0.55,1},
--     target_hue=345,target_saturation=0.55,value_scale=0.9},...}}
-- The base-colour PNG embedded in the hero's hash-pinned FBX is extracted, recoloured by the
-- lua-host png_recolor operation (only pixels inside a rule window change; layout and alpha stay),
-- and OmCharacterRecolorCommandlet imports it with a copy of the hero's imported material under
-- the variant root. The imported hero assets are never touched; the variant material reaches the
-- hero through material_sections (templates/moba_tripo_hero_visuals.lua).
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local source=require('character_source_import')
local M={schema='omoba.character-recolor-plan.v1',destination='/Game/Omoba/Characters/TripoVariantsV1',
  output='target/character-recolor',set='tripo_v1'}
-- Generated source sets that may declare colour variants, each with its own variant root.
M.sets={tripo_v1={destination=M.destination},
  tripo_static_v1={destination='/Game/Omoba/Structures/TripoStaticVariantsV1'}}
local function number(n,a,z)return type(n)=='number'and n==n and n>=a and n<=z end
local function window(w,max)return type(w)=='table'and #w==2 and number(w[1],0,max)and number(w[2],0,max)end

-- The recolour block of one manifest (nil when the hero has none); errors name the hero.
function M.validate_recolor(id,recolor)
  if recolor==nil then return nil end
  assert(type(recolor)=='table'and type(recolor.variant)=='string'and recolor.variant:match('^[a-z][a-z0-9_]*$')
    and #recolor.variant<=32,'recolor.variant must be a short identifier: '..id)
  local rules=recolor.rules;assert(type(rules)=='table'and #rules>=1 and #rules<=16,'recolor.rules must hold 1..16 rules: '..id)
  for _,r in ipairs(rules)do
    assert(window(r.hue,360)and window(r.saturation,1)and window(r.value,1)
      and r.saturation[1]<=r.saturation[2]and r.value[1]<=r.value[2],'recolor rule windows invalid: '..id)
    assert(number(r.target_hue,0,360)and number(r.target_saturation,0,1)and number(r.value_scale,0.1,2),'recolor rule targets invalid: '..id)
  end
  return recolor
end

-- Embedded textures of a binary FBX: {name=,bytes=} for every Video node with Content.
function M.embedded_textures(bytes)
  assert(bytes:sub(1,23)=='Kaydara FBX Binary  \0\26\0','binary FBX header required')
  local wide=string.unpack('<I4',bytes,24)>=7500
  local found={};local nodes=0
  local function walk(pos,limit,depth,parent,video)
    assert(depth<=64,'FBX nesting too deep')
    while pos<limit do
      local finish,props,name_len
      if wide then finish,props,_,name_len,pos=string.unpack('<I8I8I8B',bytes,pos)
      else finish,props,_,name_len,pos=string.unpack('<I4I4I4B',bytes,pos)end
      if finish==0 then return end
      finish=finish+1;assert(finish>pos and finish<=#bytes+1,'FBX node escapes file');nodes=nodes+1;assert(nodes<=2000000,'FBX node budget')
      local name=bytes:sub(pos,pos+name_len-1);pos=pos+name_len
      local values={}
      for _=1,props do
        local t=bytes:sub(pos,pos);pos=pos+1
        if t=='S'or t=='R'then local n;n,pos=string.unpack('<I4',bytes,pos);values[#values+1]={t,pos,n};pos=pos+n
        elseif t=='Y'then pos=pos+2 elseif t=='C'then pos=pos+1 elseif t=='I'or t=='F'then pos=pos+4
        elseif t=='D'or t=='L'then pos=pos+8
        elseif t:match('^[fdlibc]$')then local clen;_,_,clen,pos=string.unpack('<I4I4I4',bytes,pos);pos=pos+clen
        else error('unknown FBX property type '..string.byte(t)..' at '..pos)end
      end
      local record=video
      if name=='Video'then record={};found[#found+1]=record end
      if video and name=='Content'and values[1]and values[1][1]=='R'then
        video.bytes=bytes:sub(values[1][2],values[1][2]+values[1][3]-1)
      elseif video and(name=='RelativeFilename'or name=='Filename')and values[1]and values[1][1]=='S'then
        video.name=video.name or bytes:sub(values[1][2],values[1][2]+values[1][3]-1):match('([^/\\]+)$')
      end
      if pos<finish then walk(pos,finish,depth+1,name,record)end
      pos=finish
    end
  end
  walk(28,#bytes,0,nil,nil)
  local out={};for _,v in ipairs(found)do if v.bytes and #v.bytes>0 then out[#out+1]=v end end
  return out
end

-- The single PNG base-colour texture of a hero FBX.
function M.basecolor_png(bytes)
  local pick
  for _,t in ipairs(M.embedded_textures(bytes))do
    if t.bytes:sub(1,8)=='\137PNG\r\n\26\n'and(not pick or tostring(t.name):lower():find('basecolor',1,true))then pick=t end
  end
  return assert(pick,'FBX embeds no PNG base-colour texture')
end

-- Recolour jobs for every manifest with a recolor block; each job pins the hero FBX, the rules,
-- the imported material it copies and the derived PNG paths under OUTPUT.
function M.jobs(root,output,set_id)
  set_id=set_id or M.set;local set=assert(M.sets[set_id],'recolor does not support source set: '..tostring(set_id))
  local plan=source.validate(root,set_id);local registry=j.read(source.registry(root,set_id))
  local materials={}
  for _,row in ipairs(registry.assets or{})do for _,o in ipairs(row.objects or{})do
    if o.class=='Material'then assert(not materials[row.id],'more than one imported material: '..row.id);materials[row.id]=o.object_path end
  end end
  local jobs={}
  for _,asset in ipairs(plan.assets)do
    local m=j.read(p.join(root,source.sets[set_id].source_dir,asset.id,'manifest.json'))
    assert(m.recolor==nil or m.recolors==nil,'declare recolor or recolors, not both: '..asset.id)
    local list=m.recolors or{m.recolor};assert(type(list)=='table'and #list<=8,'recolors must hold at most 8 variants: '..asset.id)
    local variants={}
    for _,declared in ipairs(list)do local recolor=M.validate_recolor(asset.id,declared)
    assert(not variants[recolor.variant],'duplicate recolor variant: '..asset.id..' '..recolor.variant);variants[recolor.variant]=true
      assert(materials[asset.id],'imported material missing; import the hero first: '..asset.id)
      local dir=p.join(output or p.join(root,M.output),asset.id)
      jobs[#jobs+1]={id=asset.id,variant=recolor.variant,source_set=set_id,rules=recolor.rules,fbx=p.join(root,m.source),fbx_sha256=m.sha256,
        source_material=materials[asset.id],base_png=p.join(dir,asset.id..'_basecolor.png'),
        variant_png=p.join(dir,asset.id..'_basecolor_'..recolor.variant..'.png'),
        destination=set.destination..'/'..asset.id,texture_name=asset.id..'_basecolor_'..recolor.variant,
        material_name=asset.id..'_mat_'..recolor.variant}
    end
  end
  return jobs
end

-- Extracts and recolours one job's PNG through lua-host; returns the plan row for the commandlet.
function M.prepare(job,host)
  local bytes=assert(p.read(job.fbx,true));assert(h.sha256(job.fbx)==job.fbx_sha256,'hero FBX SHA256 differs: '..job.id)
  p.mkdir_p(p.parent(job.base_png));p.write(job.base_png,M.basecolor_png(bytes).bytes,true,true)
  local report=(host or require('tools.lua.lib.host')).call('png_recolor',{source=job.base_png,output=job.variant_png,rules=job.rules})
  assert(report.changed>0 and report.changed<report.pixels,'recolor changed no pixel or every pixel: '..job.id)
  return {id=job.id,variant=job.variant,source_set=job.source_set,source_material=job.source_material,texture_png=job.variant_png,
    texture_sha256=report.output_sha256,destination=job.destination,texture_name=job.texture_name,material_name=job.material_name,
    width=report.width,height=report.height,changed_pixels=report.changed,pixels=report.pixels}
end

function M.material_path(row)return row.destination..'/'..row.material_name..'.'..row.material_name end
-- Variant material object path from the set, hero/model id and variant name (for presentation templates).
function M.variant_material(set_id,id,variant)
  local name=id..'_mat_'..variant;return assert(M.sets[set_id]).destination..'/'..id..'/'..name..'.'..name
end

function M.verify_result(plan,result)
  assert(result.schema=='omoba.character-recolor-result.v1'and result.success==true and result.error=='','recolor commandlet failed')
  assert(type(result.variants)=='table'and #result.variants==#plan.variants,'wrong recolor result count')
  local by_id={};for _,row in ipairs(plan.variants)do by_id[row.id..'/'..row.variant]=row end
  local imported,verified=0,0
  for _,row in ipairs(result.variants)do
    local key=tostring(row.id)..'/'..tostring(row.variant)
    local want=assert(by_id[key],'unknown or repeated recolor result');by_id[key]=nil
    assert(row.action=='imported'or row.action=='verified','invalid recolor action')
    if row.action=='imported'then imported=imported+1 else verified=verified+1 end
    assert(row.texture==want.destination..'/'..want.texture_name..'.'..want.texture_name,'recolor texture path differs: '..row.id)
    assert(row.material==M.material_path(want),'recolor material path differs: '..row.id)
    assert(row.texture_width==want.width and row.texture_height==want.height,'recolor texture size differs: '..row.id)
    assert(type(row.replaced_samples)=='number'and row.replaced_samples>=1,'variant material samples no recoloured texture: '..row.id)
  end
  assert(imported==result.imported and verified==result.verified,'recolor counters differ')
  return result
end
return M
