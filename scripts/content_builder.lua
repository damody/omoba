-- Build/workflow-only authoring loader. Never embedded in the game runtime.
local M = {}
function M.new(content_root)
  assert(type(content_root) == 'string' and content_root ~= '', 'content root is required')
  local context, loading, depth = {}, {}, 0
  function context.include(relative)
    assert(type(relative)=='string' and relative~='' and not relative:find(':',1,true)
      and not relative:match('^[/\\]'),'include requires a content-relative path')
    local segments={}
    for segment in relative:gmatch('[^/\\]+') do
      assert(segment~='..','include rejects parent-directory escape')
      if segment~='.' then segments[#segments+1]=segment end
    end
    local canonical=table.concat(segments,'/')
    local identity=canonical:lower()
    assert(canonical~='' and not loading[identity],'include cycle: '..canonical)
    assert(depth<64,'include nesting exceeds 64')
    loading[identity]=true;depth=depth+1
    local ok,entries=xpcall(function()
      local builder=assert(loadfile(content_root..'/'..canonical))()
      assert(type(builder)=='function',canonical..' must return a builder function')
      return builder(context)
    end,debug.traceback)
    loading[identity]=nil;depth=depth-1
    assert(ok,entries)
    assert(type(entries)=='table',canonical..' builder must return a table')
    return entries
  end
  return context
end
return M
