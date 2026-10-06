local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local json=b.lib('json')
local a=b.lib('args')
local options=a.parse(arg)
local expected=json.read(a.required(options,'expected'))
local observed=json.read(a.required(options,'observed'))
local function index(rows)
  local result={}
  for _,row in ipairs(rows) do
    local key=string.format('%d:%d',row.replica_id,row.component_schema_id)
    assert(not result[key],'duplicate disclosed component key '..key)
    result[key]=row
  end
  return result
end
local left,right=index(expected),index(observed)
local keys={};for key in pairs(left) do keys[key]=true end;for key in pairs(right) do keys[key]=true end
local ordered={};for key in pairs(keys) do ordered[#ordered+1]=key end;table.sort(ordered)
local differences={}
for _,key in ipairs(ordered) do
  local x,y=left[key],right[key]
  if not x or not y or x.sha256~=y.sha256 or x.byte_len~=y.byte_len
    or x.disclosure_epoch~=y.disclosure_epoch or x.entity_kind~=y.entity_kind then
    differences[#differences+1]={key=key,expected=x,observed=y}
  end
end
local result={scope='already-disclosed-components-only',expected_count=#expected,observed_count=#observed,differences=differences}
-- File output retains disclosed payloads for local diagnosis; console emits
-- only identifiers/schema/counts, not payloads or server-only secret artifacts.
if options.output then json.write(options.output,result) end
print(string.format('disclosed components: expected=%d observed=%d differences=%d',#expected,#observed,#differences))
for _,row in ipairs(differences) do
  print(row.key..' '..(not row.expected and 'unexpected' or not row.observed and 'missing' or 'changed'))
end
