import test from "node:test";
import assert from "node:assert/strict";
import { chatAddressTarget, chatTextParts, externalChatTarget, googleMapsTarget, sharedChatLocation } from "../src/lib/chat-links.ts";

const targets = text => chatTextParts(text).flatMap(part => part.target ? [part.target] : []);
const query = target => target && new URL(target.url).searchParams.get("query");

test("URL recognition preserves Thai text, newlines, punctuation and balanced parentheses", () => {
  const text = "ดูชุด (https://example.com/Dress_(blue)).\nหรือ www.example.org/ไทย?q=สีฟ้า&size=M!";
  const parts = chatTextParts(text);
  assert.equal(parts.map(part => part.text).join(""), text);
  assert.deepEqual(targets(text).map(target => target.url), [
    "https://example.com/Dress_(blue)",
    "https://www.example.org/%E0%B9%84%E0%B8%97%E0%B8%A2?q=%E0%B8%AA%E0%B8%B5%E0%B8%9F%E0%B9%89%E0%B8%B2&size=M",
  ]);
  assert.equal(externalChatTarget("HTTP://example.com/a")?.url, "http://example.com/a");
});

test("unsafe schemes, credentials, control characters and malformed links remain inert", () => {
  for (const value of ["javascript:alert(1)", "data:text/html,test", "file:///C:/test", "intent://example.com", "//example.com", "https://user:secret@example.com", "https://exa\u202emple.com", "https://example.com/\u0000", "https://example.com/\\else", "https://", "https://example.com/has space", `https://example.com/${"x".repeat(4096)}`]) {
    assert.equal(externalChatTarget(value), null, value);
  }
  for (const text of ["user@www.example.com", "javascript:https://example.com", "https://user:secret@example.com/13.7563,100.5018", "https://exa\u202emple.com"]) {
    assert.equal(targets(text).length, 0, text);
    assert.equal(chatTextParts(text).map(part => part.text).join(""), text);
  }
  assert.equal(targets('<script>alert("test")</script>').length, 0);
});

test("known Google Maps domains are identified without trusting lookalike domains", () => {
  for (const url of ["https://maps.app.goo.gl/abc", "https://goo.gl/maps/abc", "https://www.google.com/maps/place/test", "https://maps.google.com/?q=13,100"]) assert.equal(externalChatTarget(url)?.kind, "map");
  for (const url of ["https://maps.app.goo.gl.evil.test/abc", "https://www.google.com.evil.test/maps", "https://www.google.com/search?q=maps"]) assert.equal(externalChatTarget(url)?.kind, "website");
});

test("coordinates open Maps without interpreting prices, phone numbers or URL query parameters", () => {
  const text = "พิกัด 13.7563, 100.5018\nGPS: 0, 0\n-33.8688,151.2093\nยอด 40.00, 60.00 บาท";
  assert.deepEqual(targets(text).map(query), ["13.7563, 100.5018", "0, 0", "-33.8688, 151.2093"]);
  for (const text of ["91.1234, 100.1234", "13.1234, 181.1234", "913.1234, 100.1234", "โทร 0812345678, 0891234567", "ยอด 40.00, 60.00 บาท"]) assert.equal(targets(text).length, 0, text);
  assert.equal(targets("https://example.com/?pin=13.7563,100.5018").length, 1);
});

test("native LINE pins prefer coordinates, including zero; missing pins fall back to address or title", () => {
  const pin = sharedChatLocation({ title: "ร้าน Arisa", address: "กรุงเทพ", latitude: 0, longitude: 100.5 });
  assert.equal(pin.title, "ร้าน Arisa");
  assert.equal(query(pin.target), "0, 100.5");
  assert.equal(query(sharedChatLocation({ latitude: 91, longitude: 100, address: "47 ถนนเอกชัย\nกรุงเทพ 10150" }).target), "47 ถนนเอกชัย กรุงเทพ 10150");
  assert.equal(query(sharedChatLocation({ title: "เซ็นทรัลลาดพร้าว" }).target), "เซ็นทรัลลาดพร้าว");
  for (const pin of [{}, { latitude: NaN, longitude: Infinity }, { latitude: "13.7", longitude: "100.5" }, { title: 123, address: null }]) assert.equal(sharedChatLocation(pin).target, null);
});

test("typed postal addresses exclude customer names, phone numbers and order instructions", () => {
  const text = "K.ศิริยา ตะโกดม\nบ้านเลขที่ 47 หมู่4 ซอยเอกชัย30 ถนนเอกชัย\nแขวงบางขุนเทียน เขตจอมทอง\nกทม. 10150\nTel. 087-332-3708\nค่าเช่า 600 บาท";
  assert.equal(query(chatAddressTarget(text)), "บ้านเลขที่ 47 หมู่4 ซอยเอกชัย30 ถนนเอกชัย แขวงบางขุนเทียน เขตจอมทอง กทม. 10150");
  assert.equal(query(chatAddressTarget("ที่อยู่จัดส่ง:\n90/102 หมู่บ้านโกลเด้นซิตี้ ซอย9 ต.วัดชลอ\nอ.บางกรวย จ.นนทบุรี 11130\n0959531727")), "90/102 หมู่บ้านโกลเด้นซิตี้ ซอย9 ต.วัดชลอ อ.บางกรวย จ.นนทบุรี 11130");
  assert.equal(query(chatAddressTarget("ที่อยู่: 47 ซอยเอกชัย30 แขวงบางขุนเทียน โทร 0873323708")), "47 ซอยเอกชัย30 แขวงบางขุนเทียน");
  assert.equal(query(chatAddressTarget("47 ซอยเอกชัย30 แขวงบางขุนเทียน\n087-332-3708")), "47 ซอยเอกชัย30 แขวงบางขุนเทียน");
});

test("explicit place labels support Maps searches without fabricating a precise pin", () => {
  assert.equal(query(chatAddressTarget("สถานที่: เซ็นทรัลลาดพร้าว\nขอบคุณค่ะ")), "เซ็นทรัลลาดพร้าว");
  assert.equal(query(chatAddressTarget("Address: 123 Main Street, London")), "123 Main Street, London");
  assert.equal(query(chatAddressTarget("สถานที่: Hotel Arisa")), "Hotel Arisa");
  assert.equal(query(chatAddressTarget("สถานที่: เบอร์ลิน")), "เบอร์ลิน");
  for (const text of ["ยังไม่ได้ที่อยู่ค่ะ", "ที่อยู่ไม่ถูกต้อง", "ค่าเช่า 600 บาท", "0812345678", "บัญชี 035-383304", "ชอบชุดนี้ค่ะ", "location: https://example.com", "13.7563, 100.5018"]) assert.equal(chatAddressTarget(text), null, text);
});

test("Maps queries are encoded as data and respect the supported URL length", () => {
  const target = googleMapsTarget("ร้าน & สาขา #1 / size=M");
  assert.equal(new URL(target.url).hostname, "www.google.com");
  assert.equal(new URL(target.url).searchParams.get("api"), "1");
  assert.equal(query(target), "ร้าน & สาขา #1 / size=M");
  assert.equal(new URL(target.url).hash, "");
  assert.equal(googleMapsTarget("ไทย".repeat(1000)), null);
  assert.equal(googleMapsTarget("\u202eร้าน"), null);
  assert.equal(googleMapsTarget(""), null);
});
