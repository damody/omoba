-- Strict RIFF/index and timeline audit. Actual JPEG decoding is tested natively;
-- this parser never claims a new game/ACK or wall-clock performance measurement.
local b = require('_bootstrap')
local p, j, h = b.lib('path'), b.lib('json'), b.lib('hash')
local M = {}
local MAX_BYTES, MAX_FRAMES = 1024*1024*1024, 16000
function M.inspect(bytes)
  assert(type(bytes)=='string' and #bytes>=256 and #bytes<=MAX_BYTES, 'invalid AVI size')
  local function u32(offset) assert(offset>=0 and offset+4<=#bytes,'truncated AVI integer');return string.unpack('<I4',bytes,offset+1)end
  local function four(offset) assert(offset>=0 and offset+4<=#bytes,'truncated AVI fourcc');return bytes:sub(offset+1,offset+4)end
  assert(four(0)=='RIFF' and four(8)=='AVI ' and u32(4)==#bytes-8,'invalid RIFF AVI')
  assert(four(12)=='LIST' and u32(16)==192 and four(20)=='hdrl','invalid header list')
  assert(four(24)=='avih' and u32(28)==56 and u32(32)==125000 and u32(56)==1,'invalid main header')
  local frames,width,height=u32(48),u32(64),u32(68)
  assert(frames>=1 and frames<=MAX_FRAMES and width>=1 and width<=4096 and height>=1 and height<=4096 and width*height<=8388608,'invalid frame contract')
  assert(four(88)=='LIST' and u32(92)==116 and four(96)=='strl' and four(100)=='strh' and u32(104)==56,'invalid stream list')
  assert(four(108)=='vids' and four(112)=='MJPG' and u32(128)==1 and u32(132)==8 and u32(140)==frames,'invalid MJPG timing')
  assert(four(164)=='strf' and u32(168)==40 and u32(172)==40 and u32(176)==width and u32(180)==height and four(188)=='MJPG','invalid bitmap format')
  assert(four(212)=='LIST' and four(220)=='movi','missing movie')
  local cursor,entries=224,{}
  for i=1,frames do
    assert(four(cursor)=='00dc','missing video frame')
    local size=u32(cursor+4)
    assert(size>=4 and cursor+8+size<=#bytes,'invalid JPEG length')
    assert(bytes:sub(cursor+9,cursor+10)=='\255\216' and bytes:sub(cursor+7+size,cursor+8+size)=='\255\217','invalid JPEG envelope')
    entries[i]={offset=cursor-220,size=size,position=cursor+8}
    cursor=cursor+8+size+(size%2)
  end
  assert(u32(216)==cursor-220 and four(cursor)=='idx1' and u32(cursor+4)==frames*16 and cursor+8+frames*16==#bytes,'invalid movie/index length')
  for i,entry in ipairs(entries)do
    local at=cursor+8+(i-1)*16
    assert(four(at)=='00dc' and u32(at+4)==16 and u32(at+8)==entry.offset and u32(at+12)==entry.size,'invalid keyframe index')
  end
  return {frames=frames,width=width,height=height,bytes=#bytes,entries=entries}
end
function M.verify(directory,native,source)
  source=source or 'owned-slate-window-synthetic-filtered-feed'
  assert(source=='owned-slate-window-synthetic-filtered-feed' or source=='owned-slate-window-synthetic-compiled-public-map','unknown video source')
  assert(native.record_video==true and native.video_frames>=1 and native.video_segments>=1,'native video not completed')
  local timeline=p.read(p.join(directory,'owned-window-frames.jsonl'),true)
  assert(#timeline<=16*1024*1024 and timeline:sub(-1)=='\n','invalid frame timeline')
  local segments,total,last,rows={},0,-1,{}
  for line in timeline:gmatch('([^\n]+)\n')do
    local row=j.decode(line)
    assert(math.type(row.segment)=='integer' and row.segment>=1 and row.segment<=native.video_segments,'invalid segment')
    assert(type(row.elapsed_seconds)=='number' and row.elapsed_seconds==row.elapsed_seconds and row.elapsed_seconds>=last and row.elapsed_seconds<3600,'non-monotonic time')
    assert(type(row.stage)=='string' and #row.stage>0 and row.source==source,'invalid source/stage')
    local entry=segments[row.segment]
    if not entry then
      assert(row.segment==#segments+1,'out-of-order segment')
      local name=string.format('owned-window-%03d.avi',row.segment)
      local file=p.join(directory,name);entry=M.inspect(p.read(file,true));entry.file=file;entry.sha256=h.sha256(file);entry.seen=0
      total=total+entry.bytes;assert(total<=MAX_BYTES,'total video exceeds byte budget');segments[row.segment]=entry
    end
    assert(row.frame==entry.seen and row.width==entry.width and row.height==entry.height,'frame sequence/dimensions disagree')
    entry.seen=entry.seen+1;assert(entry.seen<=entry.frames,'timeline exceeds movie');rows[#rows+1]=row;last=row.elapsed_seconds
  end
  assert(#rows==native.video_frames and #rows<=MAX_FRAMES and #segments==native.video_segments and total==native.video_bytes,'native movie totals disagree')
  for _,entry in ipairs(segments)do assert(entry.seen==entry.frames,'movie frames missing from timeline');entry.entries=nil end
  return {success=true,frames=#rows,segments=segments,bytes=total,elapsed_seconds=last,authority_ack=false,source=source,playback_fps=8,wall_clock_timeline_separate=true}
end
return M
