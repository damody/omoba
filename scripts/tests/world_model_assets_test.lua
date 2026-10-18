package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap');local j=b.lib('json');local assets=require('world_model_assets')
local plan=assets.plan();local result={schema='omoba.world-model-result.v1',success=true,error='',render_verified=false,
 imported=#plan.models,verified=0,saved=12,models={}}
for _,model in ipairs(plan.models)do result.models[#result.models+1]={id=model.id,
 path=assets.destination..'SM_'..model.id..'.SM_'..model.id,vertices=model.source_vertices,
 source_triangles=model.source_triangles,triangles=model.source_triangles,degenerate_triangles=0,bounds_cm='finite native bounds'}end
assert(assets.verify_result(plan,result));local count=1
local function reject(change)
 local value=j.decode(j.encode(result));change(value)
 assert(not pcall(assets.verify_result,plan,value),'invalid result admitted');count=count+1
end
reject(function(v)v.schema='old'end)
reject(function(v)v.success=false end)
reject(function(v)v.render_verified=true end)
reject(function(v)v.error='failure'end)
reject(function(v)v.saved=-1 end)
reject(function(v)v.saved=0.5 end)
reject(function(v)v.imported=v.imported-1 end)
reject(function(v)v.verified=-1;v.imported=v.imported+1 end)
reject(function(v)table.remove(v.models)end)
reject(function(v)v.models[2]=v.models[1]end)
reject(function(v)v.models[1].path='/Game/Foreign/SM_bad.SM_bad'end)
reject(function(v)v.models[1].vertices=v.models[1].vertices+1 end)
reject(function(v)v.models[1].source_triangles=v.models[1].source_triangles+1;v.models[1].triangles=v.models[1].triangles+1 end)
reject(function(v)v.models[1].degenerate_triangles=1 end)
reject(function(v)v.models[1].degenerate_triangles=-1;v.models[1].triangles=v.models[1].triangles+1 end)
reject(function(v)v.models[1].bounds_cm='NaN'end)
print('PASS world model source/result contracts: '..count..' cases')
