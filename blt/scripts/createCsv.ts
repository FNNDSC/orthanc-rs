const requestedTags = ['PatientID', 'PatientName', 'PatientBirthDate', 'AccessionNumber']
const uri = `http://localhost:8042/studies?expand=true&response-content=RequestedTags&requested-tags=${requestedTags.join(';')}`
const res = await fetch(uri);
const data = await res.json() as ReadonlyArray<any>;

const patientIds = new Set(data.map((study) => study.RequestedTags.PatientID));
const anonPatientIds = [...patientIds.values()]
  .map((patientId, i) => [patientId, (i + 1).toString().padStart(3, '0')])
  .reduce((map, [patientId, i]) => map.set(patientId, `BCH${i}`), new Map());

const HEADERS = ['MRN', 'Anon_PatientID', 'PatientName', 'Anon_PatientName', 'PatientBirthDate', 'Search_AccessionNumber', 'Anon_AccessionNumber', 'Anon_PatientBirthDate'];
console.log(HEADERS.join(','))

const studyCounter = new Map([...patientIds.values()].map((p) => [p, 1]));

for (const study of data) {
  const tags = study.RequestedTags;
  const anonPatientId = anonPatientIds.get(tags.PatientID);
  const visitNumber = studyCounter.get(tags.PatientID);
  studyCounter.set(tags.PatientID, visitNumber + 1);
  const anonAccessionNumber = `${anonPatientId}-visit${visitNumber.toString().padStart(2, '0')}`;
  const line = [tags.PatientID, anonPatientId, tags.PatientName, anonPatientId, tags.PatientBirthDate, tags.AccessionNumber, anonAccessionNumber, '19010101'];
  console.log(line.join(','));
}

