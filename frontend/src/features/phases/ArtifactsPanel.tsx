import React from 'react';

type Props = { data: { [k:string]: any } };
export function ArtifactsPanel({data}:Props){
  const link = (k:string)=>{
    const p = data[k];
    return p? <div><a href={p} target="_blank" rel="noreferrer">{k}</a></div> : <div>{k}: (missing)</div>;
  };
  return (
    <div>
      {link('touched_files.txt')}
      {link('dep_snapshot.txt')}
      {link('kpi.json')}
      {link('snippets.log')}
      {link('summary.json')}
      <pre style={{marginTop:8,background:'#f9fafb',padding:8}}>{JSON.stringify(data.summary||{}, null, 2)}</pre>
    </div>
  );
}
