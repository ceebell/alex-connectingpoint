// Encode uploads once for normal replies and reusable image cards.
export type ReplyImageInput = { original: string; preview: string };
async function encode(bitmap: ImageBitmap, maxEdge: number, quality: number): Promise<Blob> {
  const scale = Math.min(1, maxEdge / Math.max(bitmap.width, bitmap.height));
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(bitmap.width * scale)); canvas.height = Math.max(1, Math.round(bitmap.height * scale));
  const context = canvas.getContext("2d"); if (!context) throw new Error("canvas");
  context.fillStyle = "#fff"; context.fillRect(0, 0, canvas.width, canvas.height); context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  return new Promise((resolve, reject) => canvas.toBlob(blob => blob ? resolve(blob) : reject(new Error("encode")), "image/jpeg", quality));
}
const base64 = (blob: Blob) => new Promise<string>((resolve, reject) => {
  const reader = new FileReader(); reader.onload = () => resolve(String(reader.result).split(",", 2)[1] ?? ""); reader.onerror = () => reject(reader.error); reader.readAsDataURL(blob);
});
export async function prepareReplyImage(file: File): Promise<ReplyImageInput> {
  if (!/^image\/(jpeg|png|webp|gif)$/.test(file.type) || file.size > 40 * 1024 * 1024) throw new Error("file");
  const bitmap = await createImageBitmap(file);
  try {
    let original = await encode(bitmap, 2048, .85);
    if (original.size > 10 * 1024 * 1024) original = await encode(bitmap, 1600, .7);
    const preview = await encode(bitmap, 240, .8);
    if (original.size > 10 * 1024 * 1024 || preview.size > 1024 * 1024) throw new Error("size");
    return { original: await base64(original), preview: await base64(preview) };
  } finally { bitmap.close(); }
}
