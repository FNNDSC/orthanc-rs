import { describe, expect, it } from "bun:test";

const indexHtml = await Bun.file("./webapp/dist/index.html").text();
const scriptJs = await Bun.file("./webapp/dist/script.js").text();

/**
 * Crockford's base32 encoding of rapidhash of script.js
 */
const EXPECTED_ETAG = '"5Y1Y24E8PKPT6"';

describe("orthanc::webapp", () => {
	describe.each(["simple", "prepared"])("/%s", (base) => {
		it.each(["", "/", "/index.html"])(
			`should return index.html at "${base}%s"`,
			async (path) => {
				const res = await fetch(`http://localhost:8042/${base}${path}`);
				const actual = await res.text();
				expect(actual).toBe(indexHtml);
			},
		);
		it("should return script.js with correct MIME type", async () => {
			const res = await fetch(`http://localhost:8042/${base}/script.js`);
			expect(res.headers.get("Content-Type")).toBe("text/javascript");
			const actual = await res.text();
			expect(actual).toBe(scriptJs);
		});
	});
	describe("orthanc::webapp::prepare_bundle", () => {
		it("should have a stable ETag", async () => {
			const res = await fetch("http://localhost:8042/prepared/script.js");
			expect(res.headers.get("ETag")).toBe(EXPECTED_ETAG);
		});
		it("should return 304 Not Modified", async () => {
			const res = await fetch("http://localhost:8042/prepared/script.js", {
				headers: {
					"If-None-Match": EXPECTED_ETAG,
				},
			});
			expect(res.status).toBe(304);
		});
	});
});
