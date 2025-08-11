import { beforeAll, describe, expect, it } from "bun:test";
import Config from "../Orthanc.json";

const DICOM_FILE = "./data/image.dcm";
const DICOM_URL =
	"https://storage.googleapis.com/idc-open-data/d478b0bd-1f80-4734-8d81-47f20c36d0ab/1753337a-c390-4698-9d9b-f2a5b5a702a6.dcm";

// Reset test output file and download sample data.
beforeAll(async () => {
	await Promise.all([
		resetOutputFile(),
		downloadExampleDataIfNeeded(),
		deleteAllPatients(),
	]);
});

describe("orthanc_sdk::register_on_change", () => {
	it("should write PatientID to output file when DICOM file is received", async () => {
		const dcm = Bun.file(DICOM_FILE);
		const res = await fetch("http://localhost:8042/instances", {
			method: "POST",
			body: dcm.stream(),
			headers: {
				"Content-Type": "application/dicom",
				Expect: "",
			},
		});
		expect(res.ok).toBeTrue();

		await expectPoll(async () => {
			const outputFile = Bun.file(Config.ExampleRustPlugin.MrnFile);
			const output = await outputFile.text();
			return output.includes("MSB-01723");
		});
	});
});

describe("orthanc_sdk::register_rest_no_lock", () => {
	it("should have an API endpoint /rustexample/add which performs addition and logs the request", async () => {
		const countLogLines = () =>
			countLogLinesMatching(/example_rust_plugin:.+uri="\/rustexample\/add"/);
		const originalCount = await countLogLines();
		const res = await fetch("http://localhost:8042/rustexample/add", {
			method: "POST",
			body: JSON.stringify({ a: 3, b: 5 }),
		});
		expect(res.ok).toBeTrue();
		expect(await res.json()).toEqual({ sum: 8 });

		await expectPoll(async () => (await countLogLines()) === originalCount + 1);
	});
});

async function resetOutputFile() {
	const file = Bun.file(Config.ExampleRustPlugin.MrnFile);
	await file.write(new ArrayBuffer());
}

async function downloadExampleDataIfNeeded() {
	const file = Bun.file(DICOM_FILE);
	if (!(await file.exists())) {
		const res = await fetch(DICOM_URL);
		await file.write(res);
	}
}

async function deleteAllPatients() {
	const res = await fetch("http://localhost:8042/patients");
	expect(res.ok).toBeTrue();
	const patients = (await res.json()) as ReadonlyArray<string>;
	await Promise.all(
		patients.map((id) =>
			fetch(`http://localhost:8042/patients/${id}`, { method: "DELETE" }).then(
				(res) => expect(res.ok).toBeTrue(),
			),
		),
	);
}

async function countLogLinesMatching(re: RegExp): Promise<number> {
	const file = Bun.file("orthanc.log");
	const lines = await file.text();
	return lines.split("\n").filter((line) => re.test(line)).length;
}

async function expectPoll(fn: () => Promise<boolean>) {
	const start = Date.now();
	while (Date.now() - start < 1000) {
		if (await fn()) {
			return;
		}
		await Bun.sleep(100);
	}
	expect(await fn()).toBeTrue();
}
