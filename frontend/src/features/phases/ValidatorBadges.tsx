import React from 'react';

type Props = { summary?: { validator?: { scope?: boolean; dep?: boolean; api?: boolean|null; det?: boolean; kpi?: boolean } } };
const Badge = ({label, state}:{label:string,state:'PASS'|'FAIL'|'SKIP'}) => (
  <span style={{padding:'2px 6px',borderRadius:4,marginRight:6,background: state==='PASS'?'#d1fae5':state==='FAIL'?'#fee2e2':'#e5e7eb', color:'#111'}}>{label}: {state}</span>
);
export function ValidatorBadges({summary}:Props){
  const v = summary?.validator||{};
  const s = (b?: boolean|null)=> b===null? 'SKIP' : (b? 'PASS':'FAIL');
  return (
    <div>
      <Badge label="scope" state={s(v.scope) as any}/>
      <Badge label="dep" state={s(v.dep) as any}/>
      <Badge label="api" state={s((v.api as any)??null) as any}/>
      <Badge label="det" state={s(v.det) as any}/>
      <Badge label="kpi" state={s(v.kpi) as any}/>
    </div>
  );
}
