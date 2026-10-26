// Stickers the LINE Messaging API can send (backend models/outbound.rs keeps the same list).
// Source: https://developers.line.biz/en/docs/messaging-api/sticker-list/
export type StickerPackage = { id: string; first: number; count: number; title: string };

export const stickerPackages: StickerPackage[] = [
  { id: "11537", first: 52002734, count: 40, title: "บราวน์ โคนี่ & แซลลี่ ดุ๊กดิ๊กสเปเชียล" },
  { id: "11538", first: 51626494, count: 40, title: "CHOCO และผองเพื่อน ดุ๊กดิ๊กสเปเชียล" },
  { id: "11539", first: 52114110, count: 40, title: "UNIVERSTAR BT21 ดุ๊กดิ๊กสเปเชียล" },
  { id: "446", first: 1988, count: 40, title: "Moon: Special Edition" },
  { id: "789", first: 10855, count: 40, title: "Sally: Special Edition" },
  { id: "1070", first: 17839, count: 40, title: "Moon: Special Edition" },
  { id: "6136", first: 10551376, count: 24, title: "ตัวละคร LINE ขอโทษแทนคุณ!" },
  { id: "6325", first: 10979904, count: 24, title: "บราวน์ & โคนี่ เวอร์ชั่นตัวจิ๋ว" },
  { id: "6359", first: 11069848, count: 24, title: "บราวน์ & โคนี่ เวอร์ชั่นตัวจิ๋ว" },
  { id: "6362", first: 11087920, count: 24, title: "บราวน์ & โคนี่ เวอร์ชั่นตัวจิ๋ว" },
  { id: "6370", first: 11088016, count: 24, title: "บราวน์ & โคนี่ เวอร์ชั่นตัวจิ๋ว" },
  { id: "6632", first: 11825374, count: 24, title: "ตัวละคร LINE ขอโทษแทนคุณ!" },
  { id: "8515", first: 16581242, count: 24, title: "ตัวละคร LINE พูดจาน่ารัก★" },
  { id: "8522", first: 16581266, count: 24, title: "ตัวละคร LINE พูดจาน่ารัก★" },
  { id: "8525", first: 16581290, count: 24, title: "ตัวละคร LINE พูดจาน่ารัก★" },
];

export const stickerIds = (pack: StickerPackage) => Array.from({ length: pack.count }, (_, i) => String(pack.first + i));
export const stickerUrl = (id: string | number) => `https://stickershop.line-scdn.net/stickershop/v1/sticker/${id}/android/sticker.png`;
