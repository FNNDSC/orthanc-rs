const res = await fetch('http://localhost:8042/jobs?expand=true');
const data = await res.json() as ReadonlyArray<any>;

function count(state: string): number {
  return data.filter((job) => job.State === state).length
}

console.log(`Success ${count('Success')}  Pending ${count('Pending')}  Running ${count('Running')}  Failure ${count('Failure')}`);
