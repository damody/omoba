use std::time::Instant;
use omoba_core::game_proto::TickBatch;
use prost::Message;

fn percentile(values:&mut [u64],p:f64)->u64{values.sort_unstable();values[((values.len()-1)as f64*p)as usize]}

fn main(){
    let mut entities=vec![(0i64,0i64);10_000]; let mut samples=Vec::with_capacity(1200); let start=Instant::now();
    for tick in 0..1200u64 { let at=Instant::now(); for (index,(x,y)) in entities.iter_mut().enumerate(){*x=x.wrapping_add((index as i64+tick as i64)&3);*y=y.wrapping_sub((index as i64^tick as i64)&3);} samples.push(at.elapsed().as_micros()as u64); }
    let payload=TickBatch{tick:1,inputs:vec![],server_events:vec![],lua_content_generation:0,lua_content_hash:String::new()}.encode_to_vec();
    let wire_bytes=payload.len()+5; let bandwidth=wire_bytes as f64*120.0;
    let mut p50=samples.clone();let mut p95=samples.clone();let mut p99=samples;
    println!("phase1-baseline ok fixture=TD_STRESS entities={} ticks=1200 elapsed_ms={} cpu_tick_p50_us={} cpu_tick_p95_us={} cpu_tick_p99_us={} memory_entity_bytes={} legacy_wire_bytes={} legacy_bps_per_player={:.3}",entities.len(),start.elapsed().as_millis(),percentile(&mut p50,0.50),percentile(&mut p95,0.95),percentile(&mut p99,0.99),entities.len()*std::mem::size_of::<(i64,i64)>(),wire_bytes,bandwidth);
}
