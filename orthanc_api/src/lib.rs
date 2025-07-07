//! Orthanc API request and response types.

mod dicom;
mod job;
mod types;

pub use crate::dicom::*;
pub use crate::job::*;
pub use crate::types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use compact_str::{CompactString, ToCompactString};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_uri_by_value() {
        let id = SeriesId::new("aa8e0a3c-8ddd1186-76a07503-f1b36dbe-c009f45d");
        let actual = id.uri();
        let expected = "/series/aa8e0a3c-8ddd1186-76a07503-f1b36dbe-c009f45d";
        assert_eq!(&actual, expected)
    }

    #[test]
    fn test_uri_by_reference() {
        let id = SeriesId::new("aa8e0a3c-8ddd1186-76a07503-f1b36dbe-c009f45d");
        let id_ref = &id;
        let actual = id_ref.uri();
        let expected = "/series/aa8e0a3c-8ddd1186-76a07503-f1b36dbe-c009f45d";
        assert_eq!(&actual, expected)
    }

    #[test]
    fn test_patient_no_requested_tags() {
        let data = json!({
            "ID": "f966c646-95f12576-9d667375-ca38e459-69a8468b",
            "IsProtected": false,
            "IsStable": true,
            "Labels": [],
            "LastUpdate": "20250704T010750",
            "MainDicomTags": {
                "PatientBirthDate": "20090701",
                "PatientID": "1449c1d",
                "PatientName": "anonymized",
                "PatientSex": "M"
            },
            "Studies": [
                "e1df078e-9b2bd072-b24090ad-2ad9c9c9-16c96754"
            ],
            "Type": "Patient"
        });
        let actual: <PatientId as ResourceId>::Item = serde_json::from_value(data).unwrap();
        assert_eq!(actual.requested_tags, None);
    }

    #[test]
    fn test_series_with_requested_tags() {
        let data = json!({
            "ExpectedNumberOfInstances": null,
            "ID": "aa8e0a3c-8ddd1186-76a07503-f1b36dbe-c009f45d",
            "Instances": [
                "00aee1d3-bdb5e57d-0c229af4-7c3c303c-86bb29a8",
                "019e09fd-6606258a-b42fcd26-662166da-6fbf3d11",
                "0327cc97-723b0f33-2e349452-f27ac737-e85d27c1",
                "035b8036-4eae909d-15fc59a7-50845ab0-1826e676",
                "0770dd57-685a8ae7-4b4e2a3c-04b87da6-2bb44abe",
                "0787f689-82839efa-f483986d-04a8c586-4e6154a7",
                "09ec3f9b-572388b8-8ad309c3-9530b36b-a78d517d",
                "0c4a65d5-49198fca-ca58729e-d9b1c8e3-e417d9ac",
            ],
            "IsStable": true,
            "Labels": [],
            "LastUpdate": "20250704T010750",
            "MainDicomTags": {
                "SeriesDate": "20130308",
                "SeriesDescription": "SAG MPRAGE 220 FOV",
                "SeriesInstanceUID": "1.3.12.2.1107.5.2.19.45152.2013030808061520200285270.0.0.0",
                "SeriesNumber": "5",
                "SeriesTime": "081102.762000",
                "StationName": "AWP45152"
            },
            "ParentStudy": "e1df078e-9b2bd072-b24090ad-2ad9c9c9-16c96754",
            "RequestedTags": {
                "BodyPartExamined": "BRAIN",
                "ProtocolName": "SAG MPRAGE 220 FOV"
            },
            "Status": "Unknown",
            "Type": "Series"
        });
        type SeriesWithTags = <SeriesId as DicomResourceId<BodyPartAndProtocol>>::Item;
        let actual: SeriesWithTags = serde_json::from_value(data).unwrap();
        let expected = BodyPartAndProtocol {
            body_part_examined: "BRAIN".to_compact_string(),
            protocol_name: "SAG MPRAGE 220 FOV".to_compact_string(),
        };
        assert_eq!(actual.requested_tags, expected);
    }

    #[derive(serde::Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "PascalCase")]
    struct BodyPartAndProtocol {
        body_part_examined: CompactString,
        protocol_name: CompactString,
    }

    #[test]
    fn test_deserialize_job_retrieve_series() {
        let data = json!({
            "CompletionTime": "20250703T002648.691947",
            "Content": {
                "Description": "REST API",
                "LocalAet": "DEV",
                "Query": [
                    {
                        "0008,0050": "98edede8b2",
                        "0008,0052": "SERIES",
                        "0010,0020": "1449c1d",
                        "0020,000d": "1.2.840.113845.11.1000000001785349915.20130308061609.6346698",
                        "0020,000e": "1.3.12.2.1107.5.2.19.45152.2013030808061520200285270.0.0.0"
                    }
                ],
                "RemoteAet": "PACS",
                "TargetAet": "DEV"
            },
            "CreationTime": "20250703T002645.190848",
            "EffectiveRuntime": 3.5,
            "ErrorCode": 0,
            "ErrorDescription": "Success",
            "ErrorDetails": "",
            "ID": "0b09cfb2-d5c3-4340-9f96-0ae8812eadfe",
            "Priority": 0,
            "Progress": 100,
            "State": "Success",
            "Timestamp": "20250703T002655.833908",
            "Type": "DicomMoveScu"
        });
        let actual: JobInfo = serde_json::from_value(data).unwrap();
        let content = actual.content;
        let expected = JobContent::DicomMoveScu {
            description: CompactString::new("REST API"),
            local_aet: CompactString::new("DEV"),
            query: vec![MoveScuJobQuery::Series {
                patient_id: CompactString::new("1449c1d"),
                accession_number: CompactString::new("98edede8b2"),
                study_instance_uid: "1.2.840.113845.11.1000000001785349915.20130308061609.6346698"
                    .to_string(),
                series_instance_uid: "1.3.12.2.1107.5.2.19.45152.2013030808061520200285270.0.0.0"
                    .to_string(),
            }],
            remote_aet: CompactString::new("PACS"),
            target_aet: CompactString::new("DEV"),
        };
        assert_eq!(content, expected)
    }

    #[test]
    fn test_deserialize_resource_modification_study() {
        let value = json!({
            "CompletionTime": "20250707T134050.679818",
            "Content": {
                "Description": "REST API",
                "FailedInstancesCount": 0,
                "ID": "cfd54023-bf099aee-89d171ec-6602b46f-91058a39",
                "InstancesCount": 61,
                "IsAnonymization": true,
                "ParentResources": [
                    "11231bb6-426dbf6b-69897722-7ebab601-bd2c626c"
                ],
                "Path": "/studies/cfd54023-bf099aee-89d171ec-6602b46f-91058a39",
                "PatientID": "b9b65cc9-b7e4281d-8727a7f1-0b681068-0c2d3de8",
                "Type": "Study"
            },
            "CreationTime": "20250707T134048.977341",
            "EffectiveRuntime": 1.701,
            "ErrorCode": 0,
            "ErrorDescription": "Success",
            "ErrorDetails": "",
            "ID": "c304b4ec-43c9-418a-bebd-4f3a648015d5",
            "Priority": 0,
            "Progress": 100,
            "State": "Success",
            "Timestamp": "20250707T135101.755033",
            "Type": "ResourceModification"
        });
        let actual: JobInfo = serde_json::from_value(value).unwrap();
        let expected = ResourceModification {
            description: CompactString::new("REST API"),
            failed_instances_count: 0,
            id: StudyId::new("cfd54023-bf099aee-89d171ec-6602b46f-91058a39"),
            instances_count: 61,
            is_anonymization: true,
            parent_resources: vec![StudyId::new("11231bb6-426dbf6b-69897722-7ebab601-bd2c626c")],
            path: "/studies/cfd54023-bf099aee-89d171ec-6602b46f-91058a39".to_string(),
            patient_id: PatientId::new("b9b65cc9-b7e4281d-8727a7f1-0b681068-0c2d3de8"),
        };
        let expected_variant =
            JobContent::ResourceModification(ResourceModificationContent::Study(expected));
        assert_eq!(actual.content, expected_variant)
    }
}
