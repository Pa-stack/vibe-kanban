import React, {useState} from 'react';
import { updatePhase } from './api';

type Props = { phaseId: string };
export function TaskPhases({phaseId}:Props){
  const [tab,setTab]=useState<'prompt'|'fix'|'hardening'>('prompt');
  const [agent,setAgent]=useState('');
  const [budget,setBudget]=useState<number|''>('');
  const [allowlist,setAllow]=useState('');
  const [denylist,setDeny]=useState('');
  const save=async()=>{
    await updatePhase(phaseId,{
      type: tab,
      agent_override: agent||null,
      warm_kpi_budget: typeof budget==='number'? budget:null,
      allowlist: allowlist? JSON.parse(allowlist):[],
      denylist: denylist? JSON.parse(denylist):[],
    } as any);
  };
  const Tab = ({id,label}:{id:'prompt'|'fix'|'hardening',label:string})=> (
    <button onClick={()=>setTab(id)} style={{marginRight:8, fontWeight: tab===id?'bold':undefined}}>{label}</button>
  );
  return (
    <div>
      <div style={{marginBottom:8}}>
        <Tab id='prompt' label='Prompt'/>
        <Tab id='fix' label='Fix'/>
        <Tab id='hardening' label='Hardening'/>
      </div>
      <div style={{display:'flex',gap:12, alignItems:'center'}}>
        <input placeholder='Agent override' value={agent} onChange={e=>setAgent(e.target.value)} />
        <input placeholder='KPI budget (sec)' value={budget} onChange={e=>setBudget(e.target.value? Number(e.target.value):'')} type='number' />
        <button onClick={save}>Save Phase</button>
      </div>
      <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:8, marginTop:8}}>
        <textarea placeholder='allowlist JSON []' rows={6} value={allowlist} onChange={e=>setAllow(e.target.value)} />
        <textarea placeholder='denylist JSON []' rows={6} value={denylist} onChange={e=>setDeny(e.target.value)} />
      </div>
    </div>
  );
}
