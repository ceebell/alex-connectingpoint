import type { Metadata } from "next";
import { Mitr } from "next/font/google";
import "./globals.css";

const mitr = Mitr({ subsets: ["thai", "latin"], weight: ["300", "400", "500", "600", "700"], variable: "--font-mitr", display: "swap" });
export const metadata: Metadata = { title: "Alex | หลังบ้านแอดมิน", description: "จัดการแชทลูกค้าและเตรียมร่างออเดอร์", robots: { index: false, follow: false } };
export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="th"><body className={mitr.variable}>{children}</body></html>;
}
