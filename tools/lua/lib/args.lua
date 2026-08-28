local M = {}

function M.parse(values)
  local out = { positional = {} }
  local i = 1
  while i <= #values do
    local value = values[i]
    if value:sub(1, 2) == "--" then
      local name = value:sub(3)
      assert(name ~= "", "empty option name")
      local next_value = values[i + 1]
      if next_value == nil or next_value:sub(1, 2) == "--" then
        out[name] = true
      else
        out[name] = next_value
        i = i + 1
      end
    else
      table.insert(out.positional, value)
    end
    i = i + 1
  end
  return out
end

function M.integer(value, name, minimum, maximum)
  local number = math.tointeger(tonumber(value))
  assert(number, (name or "value") .. " must be an integer")
  assert(not minimum or number >= minimum, (name or "value") .. " is below minimum")
  assert(not maximum or number <= maximum, (name or "value") .. " is above maximum")
  return number
end

function M.choice(value, name, allowed)
  for _, candidate in ipairs(allowed) do
    if value == candidate then return value end
  end
  error((name or "value") .. " must be one of: " .. table.concat(allowed, ", "))
end

function M.required(options, name)
  local value = options[name]
  assert(value ~= nil and value ~= true and value ~= "", "missing --" .. name)
  return value
end

return M
