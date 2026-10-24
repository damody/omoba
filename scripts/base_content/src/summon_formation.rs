//! Strict shared formation admission. All metadata/geometry precedes mutation.
use omb_script_abi::types::{Angle,Fixed64,Vec2};
use omoba_core::ability_meta::AbilityLevelData;
use omoba_template_ids::{active_ability_const,AbilityId};

pub(crate) struct Formation {pub count:u16,pub rows:u16,pub duration:Fixed64,row:Fixed64,col:Fixed64,front:Fixed64}
pub(crate) fn resolve(id:AbilityId,rank:u8,json:&str,rows:u16)->Result<Formation,&'static str> {
 let content=active_ability_const(id).ok_or("missing formation content")?;
 let index=usize::from(rank.checked_sub(1).ok_or("invalid formation rank")?);
 if rank>content.max_level||index>=content.levels.len()||json.len()>65_536||rows==0||rows>128 {return Err("invalid formation admission");}
 let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid formation metadata")?;
 let read=|key:&str,max:i32,positive:bool|->Result<Fixed64,&'static str> {
  let value=if let Some(value)=data.extra.get(key) {
   let value=value.as_f64().ok_or("invalid formation extra type")?;
   if !value.is_finite()||value<0.0||value>f64::from(max)||(value>0.0&&value<1.0/1024.0)
    ||(key=="summon_count"&&value.fract()!=0.0) {return Err("invalid formation scalar");}
   Fixed64::from_raw((value*1024.0).round() as i64)
  }else{content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,values)|values.get(index))
   .copied().ok_or("missing formation extra")?};
  if value<Fixed64::ZERO||value>Fixed64::from_i32(max)||(positive&&value==Fixed64::ZERO) {return Err("invalid formation bounds");}
  Ok(value)
 };
 let count=read("summon_count",128,true)?;
 if count.raw()%1024!=0 {return Err("fractional formation count");}
 let count=(count.raw()/1024) as u16;
 if count<2||count%rows!=0 {return Err("formation count must fill rows");}
 let result=Formation{count,rows,duration:read("duration",3600,true)?,row:read("row_spacing",10000,true)?,
  col:read("col_spacing",10000,true)?,front:read("front_row_distance",10000,false)?};
 if result.front+result.row*Fixed64::from_i32(i32::from(rows)-1)>Fixed64::from_i32(10000)
  ||result.col*Fixed64::from_i32(i32::from(count/rows)-1)*Fixed64::from_raw(512)>Fixed64::from_i32(10000)
  {return Err("formation extent exceeds budget");}
 Ok(result)
}
impl Formation {
 pub fn positions(&self,origin:Vec2,facing:Angle)->Result<Vec<Vec2>,&'static str> {
  let limit=Fixed64::from_i32(1_000_000);
  if origin.x>limit||origin.x< -limit||origin.y>limit||origin.y< -limit {return Err("formation origin outside world budget");}
  let (x,y)=(omoba_sim::trig::cos(facing),omoba_sim::trig::sin(facing));
  let cols=self.count/self.rows;let center=Fixed64::from_i32(i32::from(cols)-1)*Fixed64::from_raw(512);
  let mut result=Vec::with_capacity(usize::from(self.count));
  for row in 0..self.rows {let front=self.front+self.row*Fixed64::from_i32(i32::from(row));
   for col in 0..cols {let side=(Fixed64::from_i32(i32::from(col))-center)*self.col;
    let at=Vec2{x:origin.x+x*front-y*side,y:origin.y+y*front+x*side};
    if at.x>limit||at.x< -limit||at.y>limit||at.y< -limit {return Err("formation leaves world budget");}
    result.push(at);
   }
  }
  Ok(result)
 }
}
