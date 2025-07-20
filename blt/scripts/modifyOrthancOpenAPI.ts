import spec from '../../orthanc_sdk/3rdparty/orthanc-openapi.json';

spec.openapi = process.env.OPENAPI_VERSION ?? '3.0.4';
spec.info.description = `
Orthanc API specification modified from v1.12.8 obtained from https://orthanc.uclouvain.be/api/.

Its differences include:

- Demo server removed
- Basic authentication is required
- BLT plugin API endpoint
`;

spec.servers = [];
spec.components = {
  securitySchemes: {
    basicAuth: {
      type: 'http',
      scheme: 'Basic'
    }
  },
  schemas: {
    BltStudy: {
      type: 'object',
      required: [
        'MRN',
        'Anon_PatientID',
        'PatientName',
        'Anon_PatientName',
        'PatientBirthDate',
        'Search_AccessionNumber',
        'Anon_AccessionNumber',
        'Anon_PatientBirthDate'
      ],
      properties: {
        'MRN': { type: 'string' },
        'Anon_PatientID': { type: 'string' },
        'PatientName': { type: 'string' },
        'Anon_PatientName': { type: 'string' },
        'PatientBirthDate': { type: 'string' },
        'Search_AccessionNumber': { type: 'string' },
        'Anon_AccessionNumber': { type: 'string' },
        'Anon_PatientBirthDate': { type: 'string' }
      }
    }
  }
};

spec.security = [
  { basicAuth: [] }
];

spec.paths['/blt/studies'] = {
  post: {
    operationId: 'importNewBltStudy',
    summary: 'Request new BLT study',
    description: 'Request to query, retrieve, anonymize, then push a DICOM study to BLT',
    tags: ['Custom BLT Plugin'],
    parameters: [],
    requestBody: {
      required: true,
      description: 'Study details and anonymization parameters',
      content: {
        'application/json': {
          schema: {
            '$ref': '#/components/schemas/BltStudy'
          }
        }
      }
    },
    responses: {
      '201': {
        description: 'Study was found by PACS query. A PACS retrieve job is scheduled.',
        content: {
          'application/json': {
            schema: {
              type: 'object',
              properties: {
                QueryID: { type: 'string', format: 'uuid' },
                JobID: { type: 'string', format: 'uuid' }
              },
              required: ['QueryID', 'JobID']
            }
          }
        }
      }
    },
  },
  get: {
    operationId: 'getBltStates',
    summary: 'Get BLT study states',
    description: 'Get the state of previously requested BLT studies, i.e. their PACS retrieve and anonymization job IDs.',
    tags: ['Custom BLT Plugin'],
    parameters: [],
    responses: {
      '200': {
        description: 'BLT study states',
        content: {
          'application/json': {
            schema: {
              type: 'array',
              items: {
                type: 'object',
                properties: {
                  Info: { '$ref': '#/components/schemas/BltStudy' },
                  QueryID: { type: 'string', format: 'uuid' },
                  RetrieveJobID: { type: 'string', format: 'uuid' },
                  AnonymizationJobID: { type: 'string', format: 'uuid' },
                  PushJobID: { type: 'string', format: 'uuid' },
                },
                required: ['info', 'query_id', 'retrieve_job_id']
              }
            }
          }
        }
      }
    }
  }
};

console.log(JSON.stringify(spec, undefined, 2));

