-- Stops version control tracking the assets built from text: they come back from DShader/, DFX/ and the images fx_family.lua lists
-- (dream_assets.lua ensure, the first asset step of build_ue_moba). History is untouched and the files stay on disk.
--
--   tools\lua\lua.exe scripts\dream_untrack.lua           what would change
--   tools\lua\lua.exe scripts\dream_untrack.lua --apply   write the .gitignore block and git rm --cached, staged only: commit them yourself
--
-- Safe to run again after more assets become text.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

M.BEGIN='# Built from text (scripts/dream_untrack.lua): run build_ue_moba after a clone'
M.END='# End of assets built from text'
-- Everything directly in these is text, so a new asset there needs no new line.
M.WHOLE={'Content/Omoba/VFX/Fx','Content/Omoba/VFX/Fx/Instances','Content/Omoba/VFX/Fx/Systems','Content/Omoba/VFX/Fx/Textures'}

-- The .gitignore text with the block replaced.
function M.ignore_text(text,files)
  local nl=text:find('\r\n',1,true) and '\r\n' or '\n'
  local lines={M.BEGIN}
  local whole={}
  for _,dir in ipairs(M.WHOLE) do whole[dir]=true;lines[#lines+1]=dir..'/*.uasset' end
  for _,file in ipairs(files) do
    if not whole[file:match('^(.*)/')] then lines[#lines+1]=file end
  end
  lines[#lines+1]=M.END
  local block=table.concat(lines,nl)..nl
  local first=text:find(M.BEGIN,1,true)
  if first then
    local _,last=text:find(M.END,first,true)
    local tail=text:sub(last+1):gsub('^\r?\n','')
    return text:sub(1,first-1)..block..tail
  end
  return text:gsub('%s*$','')..nl..nl..block
end

function M.main(argv)
  local b=require('_bootstrap')
  local p,r=b.lib('path'),b.lib('process')
  local lfs=require('lfs')
  local omfue=(p.join(b.root,'omfue'):gsub('\\','/'))
  local files={}
  for _,file in ipairs(require('dream_clone_verify').text_owned(lfs,omfue)) do files[#files+1]=file:sub(#omfue+2) end
  local known={}
  for _,row in ipairs(require('fx_family').image_assets()) do files[#files+1]=row.asset;known[row.asset]=true end
  table.sort(files)
  local missing={}
  for _,file in ipairs(files) do if not lfs.attributes(omfue..'/'..file) then missing[#missing+1]=file end end
  assert(#missing==0,'no file on disk, run dream_assets.lua ensure first:\n'..table.concat(missing,'\n'))
  local function git(args)
    local result=r.run('git',args,{cwd=omfue,check=false,label='git '..args[1]})
    assert(result.exit_code==0,'git '..args[1]..': '..tostring(result.stderr))
    return result.stdout or ''
  end
  local tracked={}
  for line in git({'ls-files','Content'}):gmatch('[^\r\n]+') do tracked[line]=true end
  -- The Textures line ignores the whole folder: an image without a source would be lost.
  for line in pairs(tracked) do
    assert(not line:match('^Content/Omoba/VFX/Fx/Textures/') or known[line],'not in fx_family.lua: '..line)
  end
  local list={}
  for _,file in ipairs(files) do if tracked[file] then list[#list+1]=file end end
  print(('%d assets built from text, %d still tracked'):format(#files,#list))
  if argv[1]~='--apply' then return 0 end
  assert(#argv==1,'unknown argument: '..tostring(argv[2]))
  local ignore=omfue..'/.gitignore'
  p.write(ignore,M.ignore_text(p.read(ignore,true),files),true,true)
  if #list>0 then
    local pathspec=p.join(b.root,'target/dream-untrack.txt')
    p.write(pathspec,table.concat(list,'\n')..'\n',true,true)
    git({'rm','--cached','--quiet','--pathspec-from-file='..pathspec})
  end
  for _,file in ipairs(files) do assert(lfs.attributes(omfue..'/'..file),'gone from disk: '..file) end
  local ignored=0
  for line in git({'ls-files','--others','--ignored','--exclude-standard','Content'}):gmatch('[^\r\n]+') do
    ignored=ignored+1
  end
  print(('staged %d removals; %d files under Content are ignored'):format(#list,ignored))
  return 0
end

if arg and arg[0] and arg[0]:match('dream_untrack%.lua$') then
  local ok,code=xpcall(M.main,debug.traceback,arg)
  if not ok then io.stderr:write(tostring(code)..'\n');os.exit(1) end
  os.exit(code)
end
return M
