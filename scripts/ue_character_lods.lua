-- LOD copies of the character bodies and their attachments (OmCharacterLods):
-- /Game/Omoba/Characters/TripoLodV1/SK_<body> and SM_<attachment>. The imported packages stay as registered.
--   tools/lua/lua.exe scripts/ue_character_lods.lua --ue-root D:/UE5.8 --run-id FRESH
-- Runs the commandlet twice from a binaries snapshot (E116); the second pass must build nothing, and every
-- level the runtime forces has to stay within its triangle budget.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,r=b.lib('path'),b.lib('json'),b.lib('process')
local M={}
-- Budgets on the game camera (triangles): hero body LOD1 (Quaternius bodies also wear a head),
-- minion or jungle LOD2, attachment LOD1.
M.budget={tripo_lod1=8000,quaternius_lod1=6600,lod2=3000,attachment_lod1=1500}
local function folders(set)
 local ids={};local base=p.join(b.root,'omfue/Content/Omoba/Characters',set)
 for name in require('lfs').dir(base)do
  if name:match('^[%w_]+$')and p.exists(p.join(base,name,name..'.uasset'))then ids[#ids+1]='Characters/'..set..'/'..name..'/'..name end
 end
 table.sort(ids);return ids
end
function M.bodies()
 local ids=folders('TripoV1')
 for _,id in ipairs(folders('QuaterniusV2'))do if not id:find('animation_library',1,true)then ids[#ids+1]=id end end
 return ids
end
function M.attachments()
 local ids={}
 for name in require('lfs').dir(p.join(b.root,'omfue/Content/Omoba/Structures/TripoStaticV1'))do
  if name:match('^weapon_[%w_]+$')then ids[#ids+1]='Structures/TripoStaticV1/'..name..'/'..name end
 end
 for _,name in ipairs{'male_head','male_head_neck','female_head','female_head_neck'}do ids[#ids+1]='Characters/RigidPartsV1/'..name end
 table.sort(ids);return ids
end
function M.check_budgets(rows)
 for _,row in ipairs(rows)do local t=row.lod_triangles
  if row.kind=='attachment'then
   assert(t[2]<=M.budget.attachment_lod1,row.id..' LOD1 is over budget: '..t[2])
  else
   local lod1=row.mesh:find('/SK_',1,true)and(M.quaternius[row.id]and M.budget.quaternius_lod1 or M.budget.tripo_lod1)
   assert(t[2]<=lod1,row.id..' LOD1 is over budget: '..t[2]);assert(t[3]<=M.budget.lod2,row.id..' LOD2 is over budget: '..t[3])
  end
 end
end

function M.main(args)
 local engine,id;local i=1
 while i<=#args do
  if args[i]=='--ue-root'then i=i+1;engine=assert(args[i])
  elseif args[i]=='--run-id'then i=i+1;id=assert(args[i])
  elseif args[i]=='--help'then print('Usage: tools/lua/lua.exe scripts/ue_character_lods.lua --ue-root PATH --run-id FRESH');return
  else error('unknown argument: '..tostring(args[i]))end;i=i+1
 end
 engine=p.absolute(assert(engine,'--ue-root is required'),b.root)
 assert(type(id)=='string'and id:match('^[%w_-]+$'),'fresh --run-id required')
 local bodies,attachments=M.bodies(),M.attachments()
 M.quaternius={};for _,path in ipairs(bodies)do if path:find('/QuaterniusV2/',1,true)then M.quaternius[path:match('[^/]+$')]=true end end
 local out=p.join(b.root,'omfue/Saved/Automation/CharacterLods',id);assert(not p.exists(out),'fresh evidence directory required');p.mkdir_p(out)
 local snapshot=require('ue_run_snapshot').prepare(b.root)
 require('ue_binary_preflight').require_ready(engine,snapshot.uproject)
 local function once(label)
  local result_file=p.join(out,label..'-result.json')
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{snapshot.uproject,'-run=OmCharacterLods',
   '-bodies='..table.concat(bodies,','),'-attachments='..table.concat(attachments,','),'-result='..result_file,
   '-unattended','-nop4','-nosplash','-nopause','-NullRHI','-NoSound','-NoLiveCoding',
   '-abslog='..p.join(out,label..'-engine.log')},{cwd=snapshot.project_dir,check=false})
  p.write(p.join(out,label..'-process.log'),(result.stdout or'')..(result.stderr or''),false,true)
  assert(result.exit_code==0,label..' character LOD commandlet failed; see '..out)
  local value=j.read(result_file)
  assert(value.schema=='omoba.character-lods/1'and value.success==true,label..' result is not a success')
  assert(#value.meshes==#bodies+#attachments,label..' handled '..#value.meshes..' of '..(#bodies+#attachments))
  return value
 end
 local first=once('first');local second=once('second')
 assert(second.created==0,'repeat pass rebuilt meshes')
 for _,row in ipairs(first.meshes)do print(row.kind,row.id,table.concat(row.lod_triangles,' / '))end
 M.check_budgets(first.meshes)
 print(('Character LODs PASS: %d created, %d verified, repeat built 0; evidence: %s'):format(first.created,first.verified,out))
 return first
end
if ...=='ue_character_lods'then return M end
M.main(arg)
