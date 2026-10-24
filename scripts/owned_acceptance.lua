-- Fixed-Lua acceptance, with actual identity-bound exit evidence.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local session=require('moba_session_process');local M={schema='omoba-owned-acceptance/1'}
local function array(value,label,limit)
 value=value or {};assert(type(value)=='table'and getmetatable(value)==nil,label..' must be an array')
 local n=0;for k in pairs(value)do assert(math.type(k)=='integer'and k>=1 and k<=limit,label..' index');n=n+1 end
 assert(#value==n,label..' sparse array');local result={}
 for i=1,n do assert(type(value[i])=='string'and not value[i]:find('\0',1,true),label..' string');result[i]=value[i]end
 return result
end
function M.validate(value)
 assert(type(value)=='table'and getmetatable(value)==nil,'plain acceptance spec required')
 local fields={schema=true,script=true,args=true,output=true,summary=true,watch=true}
 for k in pairs(value)do assert(fields[k],'unknown acceptance field: '..tostring(k))end
 assert(value.schema==M.schema,'unknown acceptance schema')
 assert(type(value.script)=='string'and value.script:match('%.lua$'),'Lua script required')
 assert(type(value.output)=='string'and value.output~='','exclusive output directory required')
 local spec={script=p.absolute(value.script,b.root),output=p.absolute(value.output,b.root),args=array(value.args,'args',4096),watch={}}
 assert(p.is_file(spec.script),'missing Lua script')
 if value.summary~=nil then assert(type(value.summary)=='string'and value.summary~='','summary path');spec.summary=p.absolute(value.summary,b.root)end
 for _,file in ipairs(array(value.watch,'watch',64))do spec.watch[#spec.watch+1]=p.absolute(file,b.root)end
 return spec
end
function M.verify_exit(identity,receipt,process)
 local verified=process.validate_owned_exit(identity,receipt)
 return verified.exit_code==0,verified
end
function M.run(value,process,time)
 local spec=M.validate(value)
 assert(not p.exists(spec.output),'acceptance output already exists')
 assert(not spec.summary or not p.exists(spec.summary),'summary must be fresh before spawn')
 p.mkdir_p(p.parent(spec.output));assert(require('lfs').mkdir(spec.output),'exclusive acceptance directory creation failed')
 local records,entry,result={},nil,nil;local started=time.utc_timestamp();local fingerprint=h.sha256(spec.script)
 local stdout,stderr=p.join(spec.output,'stdout.log'),p.join(spec.output,'stderr.log')
 local health=p.join(spec.output,'health.jsonl');local last_sample,max_gap,samples=nil,0,0
 local function sample(final)
  local now=time.monotonic_ms();if last_sample then max_gap=math.max(max_gap,now-last_sample)end;last_sample=now;samples=samples+1
  local files={};local paths={stdout,stderr};for _,file in ipairs(spec.watch)do paths[#paths+1]=file end
  for _,file in ipairs(paths)do if p.is_file(file)then local a=p.attributes(file);files[#files+1]={file=file,bytes=a.size,mtime=a.modification}end end
  p.append(health,j.encode{utc=time.utc_timestamp(),final=final,files=files}..'\n')
 end
 local ok,err=xpcall(function()
  local args={spec.script};for _,text in ipairs(spec.args)do args[#args+1]=text end
  sample(false)
  entry=session.spawn('acceptance',{exe=b.lib('platform').lua_executable,args=args,cwd=b.root,stdout=stdout,stderr=stderr},p.join(spec.output,'process'),process,time,
   function(identity,label)records[#records+1]={identity=identity,label=label};j.write(p.join(spec.output,'identities.json'),{records=records},true)end)
  sample(false);local due=time.monotonic_ms()+140000
  while not process.wait_owned(entry.child,30000)do
   if time.monotonic_ms()>=due then sample(false);due=time.monotonic_ms()+140000 end
  end
  sample(false)
  result=session.exit(entry,process)
  local successful;successful,result=M.verify_exit(entry.child,result,process)
  assert(successful,'acceptance child exited with Windows code '..result.exit_code)
  assert(h.sha256(spec.script)==fingerprint,'acceptance runner changed while executing')
  if spec.summary then
   assert(p.is_file(spec.summary)and p.attributes(spec.summary).size<=1048576,'missing or oversized final summary')
   local summary=j.read(spec.summary);assert(type(summary)=='table'and summary.success==true,'final summary did not succeed')
  end
 end,debug.traceback)
 local clean=true
 for i=#records,1,-1 do local identity=records[i].identity;local retired,why=pcall(function()
  if process.owned_alive(identity)then process.stop_owned(identity)end
  assert(process.wait_owned(identity,5000)and not process.owned_alive(identity),'original process not retired')
 end);if not retired then clean=false;err=(err or '')..'\n'..tostring(why)end end
 sample(true)
 local receipt={schema='omoba-owned-acceptance-result/1',success=ok and clean,retired=clean,cleanup_succeeded=clean,
  started_utc=started,completed_utc=time.utc_timestamp(),script=spec.script,script_sha256=fingerprint,
  owner=entry and entry.child or nil,waiter=entry and entry.waiter or nil,exit_code=result and result.exit_code or nil,
  exit_receipt_file=entry and entry.exit_file or nil,summary=spec.summary,error=(ok and clean)and ''or tostring(err),health={samples=samples,max_gap_ms=max_gap}}
 j.write(p.join(spec.output,'receipt.json'),receipt,false)
 if not receipt.success then p.write(p.join(spec.output,'errors.md'),'# Acceptance failed\n\n'..receipt.error..'\n\nOnly recorded original lifetimes were retired. Intermediate log text and final summary cannot override a failed Windows exit.\n',false)end
 return receipt
end
return M
