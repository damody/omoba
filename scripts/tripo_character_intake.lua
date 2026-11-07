-- Intake of one downloaded Tripo hero rig into omfue/SourceArt/Characters/TripoV1/<hero>/.
-- The download is untrusted data: it is only read (header, size, node walk for a bone estimate),
-- copied byte-for-byte to <hero>/<hero>.fbx and hash-pinned in that directory's manifest.json
-- (schema omoba.tripo-character-source.v1, validated by character_source_import.lua). Nothing in
-- the file is executed. The Grok reference images already in <hero>/reference/ are hash-listed.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash');local lfs=require('lfs')
local M={}

-- Counts FBX Model nodes of class LimbNode (skeleton joints) in a binary FBX. Unreal may add the
-- armature root, so this is an estimate the import verifies; --bones overrides it.
function M.limb_nodes(bytes)
  assert(bytes:sub(1,23)=='Kaydara FBX Binary  \0\26\0','binary FBX header required')
  local version=string.unpack('<I4',bytes,24);local wide=version>=7500
  local count,nodes=0,0
  local function walk(pos,limit,depth)
    assert(depth<=64,'FBX nesting too deep')
    while pos<limit do
      local finish,props,_,name_len
      if wide then finish,props,_,name_len,pos=string.unpack('<I8I8I8B',bytes,pos)
      else finish,props,_,name_len,pos=string.unpack('<I4I4I4B',bytes,pos)end
      if finish==0 then return end
      finish=finish+1 -- EndOffset is a 0-based absolute file offset
      assert(finish>pos and finish<=#bytes+1,'FBX node escapes file');nodes=nodes+1;assert(nodes<=2000000,'FBX node budget')
      local name=bytes:sub(pos,pos+name_len-1);pos=pos+name_len
      local strings={}
      for _=1,props do
        local t=bytes:sub(pos,pos);pos=pos+1
        if t=='S'or t=='R'then local n;n,pos=string.unpack('<I4',bytes,pos);strings[#strings+1]=bytes:sub(pos,pos+n-1);pos=pos+n
        elseif t=='Y'then pos=pos+2 elseif t=='C'then pos=pos+1 elseif t=='I'or t=='F'then pos=pos+4
        elseif t=='D'or t=='L'then pos=pos+8
        elseif t:match('^[fdlibc]$')then local _,_,clen;_,_,clen,pos=string.unpack('<I4I4I4',bytes,pos);pos=pos+clen
        else error('unknown FBX property type '..string.byte(t)..' at '..pos)end
      end
      if name=='Model'and strings[#strings]=='LimbNode'then count=count+1 end
      if pos<finish then walk(pos,finish,depth+1)end
      pos=finish
    end
  end
  walk(28,#bytes,0)
  return count
end

local function main()
  local hero,download,task,version,credits,scale,bones,replace,static;local options={}
  local i=1;while i<=#arg do
    local a=arg[i]
    if a=='--hero'then i=i+1;hero=arg[i]
    elseif a=='--download'then i=i+1;download=arg[i]
    elseif a=='--task-id'then i=i+1;task=arg[i]
    elseif a=='--model-version'then i=i+1;version=arg[i]
    elseif a=='--credits'then i=i+1;credits=math.tointeger(tonumber(arg[i]))
    elseif a=='--import-scale'then i=i+1;scale=tonumber(arg[i])
    elseif a=='--bones'then i=i+1;bones=math.tointeger(tonumber(arg[i]))
    elseif a=='--option'then i=i+1;local k,v=assert(arg[i]):match('^([%w_]+)=(.*)$');assert(k,'--option key=value');options[k]=v
    elseif a=='--replace'then replace=true
    elseif a=='--static'then static=true -- unrigged world model (tripo_static_v1): no skeleton
    elseif a=='--help'then print('tools/lua/lua.exe scripts/tripo_character_intake.lua --hero ID --download FILE.fbx --task-id ID --model-version V --credits N [--option key=value]... [--import-scale X] [--bones N] [--static] [--replace]');return
    else error('unknown intake argument: '..tostring(a))end;i=i+1
  end
  assert(type(hero)=='string'and hero:match('^[a-z][a-z0-9_]*$'),'--hero id required')
  assert(download and p.is_file(download),'--download file required');assert(task and task~='','--task-id required')
  assert(version and version~='','--model-version required');assert(credits and credits>=0,'--credits integer required')
  local set_dir=static and'omfue/SourceArt/Structures/TripoStaticV1'or'omfue/SourceArt/Characters/TripoV1'
  local dir=p.join(b.root,set_dir,hero);local target=p.join(dir,hero..'.fbx');local manifest=p.join(dir,'manifest.json')
  assert(replace or not p.exists(target),'hero rig already present; pass --replace to supersede it (record why in asset-ledger.md)')
  local size=p.attributes(download).size;assert(size>=27 and size<=64*1024*1024,'bounded FBX required')
  local bytes=assert(p.read(download,true));local estimate=M.limb_nodes(bytes)
  assert(not static or estimate==0,'static world model must carry no skeleton joints')
  p.write(target,bytes,true,true)
  local refs={}
  local ref_dir=p.join(dir,'reference')
  if p.is_directory(ref_dir)then for name in lfs.dir(ref_dir)do if name:match('%.png$')then
    refs[#refs+1]={file='reference/'..name,sha256=h.sha256(p.join(ref_dir,name)),generator='Grok Build CLI image_gen',prompt='reference/'..name:gsub('%.png$','.prompt.txt')}
  end end end
  table.sort(refs,function(x,y)return x.file<y.file end);assert(#refs>=1,'reference images required in '..ref_dir)
  local value={schema=static and'omoba.tripo-static-source.v1'or'omoba.tripo-character-source.v1',id=hero,source=set_dir..'/'..hero..'/'..hero..'.fbx',
    sha256=h.sha256(target),bones=static and 0 or bones or estimate,animations=0,import_scale=scale,
    provenance={service='tripo3d.ai',task_id=task,model_version=version,options=options,credits_spent=credits,
      downloaded_at=os.date('!%Y-%m-%dT%H:%M:%SZ'),download_name=download:match('[^/\\]+$'),download_sha256=h.sha256(download),
      fbx_limb_nodes=estimate,reference_images=refs}}
  j.write(manifest,value,true)
  print(('Intake %s: %d bytes, %d LimbNode joints, manifest %s'):format(hero,size,estimate,manifest))
end

if arg and arg[0]and arg[0]:match('tripo_character_intake%.lua$')then main()end
return M
