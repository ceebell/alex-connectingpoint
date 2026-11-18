import assert from 'node:assert/strict';
import test from 'node:test';
import { compareMessages, mergeMessages, reconcileMessages, refreshChatWindow } from '../src/lib/chat-history.ts';

const row = n => ({ _id:n.toString(16).padStart(24,'0'), event_id:`event-${String(n).padStart(6,'0')}`, event_at:new Date(n*1000).toISOString(), status:'DONE' });
function history(records) {
  let calls=0;
  return { load:async before => {
    calls++; const index=before ? records.findIndex(item=>item._id===before) : records.length;
    assert(index>=0,'cursor must belong to the selected chat');
    const items=records.slice(Math.max(0,index-200),index);
    return {batches:[items],next:null,previous:index>200 ? items[0]._id : null};
  }, calls:()=>calls };
}

test('initial chat fetch returns the newest window without downloading all history',async()=>{
  const records=Array.from({length:650},(_,n)=>row(n)); const api=history(records);
  const result=await refreshChatWindow(api.load);
  assert.equal(result.records.length,200); assert.equal(result.records[0]._id,row(450)._id); assert.equal(result.records.at(-1)._id,row(649)._id); assert.equal(api.calls(),1);
});

test('refresh drains all newly arrived pages while preserving the opened history boundary',async()=>{
  const records=Array.from({length:1250},(_,n)=>row(n)); const api=history(records);
  const result=await refreshChatWindow(api.load,row(450));
  assert.equal(result.records.length,800); assert.equal(result.records[0]._id,row(450)._id); assert.equal(result.records.at(-1)._id,row(1249)._id); assert.equal(result.previous,row(450)._id); assert.equal(api.calls(),4);
});

test('one new message does not silently open two hundred older messages',async()=>{
  const api=history(Array.from({length:651},(_,n)=>row(n)));
  const result=await refreshChatWindow(api.load,row(450));
  assert.equal(result.records.length,201); assert.equal(result.previous,row(450)._id);
});

test('a withdrawn boundary cannot cause the refresh loop to miss later rows',async()=>{
  const records=Array.from({length:651},(_,n)=>row(n)).filter(item=>item._id!==row(450)._id);
  const result=await refreshChatWindow(history(records).load,row(450));
  assert.equal(result.records.length,200); assert.equal(result.previous,row(451)._id);
});

test('refresh removes withdrawn rows, includes late captures and preserves a concurrent local reply',async()=>{
  const snapshot=[row(1),row(3),row(4)], pending=row(6);
  const result=reconcileMessages([...snapshot,pending],snapshot,[row(1),row(2),row(4),row(5)]);
  assert.deepEqual(result.map(item=>item._id),[1,2,4,5,6].map(n=>row(n)._id));
});

test('identical timestamps and event keys have a deterministic ID tie breaker',()=>{
  const a=row(1),b={...row(2),event_id:a.event_id,event_at:a.event_at};
  assert(compareMessages(a,b)<0); assert.deepEqual(mergeMessages([b],[a,b]),[a,b]);
});

test('a broken cursor fails explicitly instead of looping indefinitely',async()=>{
  let count=0;
  await assert.rejects(()=>refreshChatWindow(async()=>{count++;return {batches:[[row(50)]],next:null,previous:'stuck'}},row(1)),/did not advance/);
  assert.equal(count,2);
});
