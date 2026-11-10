import type { Metadata, Viewport } from "next";
import { Mitr } from "next/font/google";
import "./globals.css";
import "./workspace-layout.css";
import "./mobile.css";
import "./typography.css";
import "../components/workspace-sidebar.css";
import "../components/date-input.css";
import "../components/reply-composer.css";
import "../components/customer-notes.css";
import "../components/team.css";

const mitr = Mitr({ subsets: ["thai", "latin"], weight: ["300", "400", "500", "600", "700"], variable: "--font-mitr", display: "swap" });
export const metadata: Metadata = { title: "Alex | หลังบ้านแอดมิน", description: "จัดการแชทลูกค้าและเตรียมร่างออเดอร์", robots: { index: false, follow: false } };
export const viewport: Viewport = { width: "device-width", initialScale: 1, viewportFit: "cover", interactiveWidget: "resizes-content" };
export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="th"><body className={mitr.variable}>{children}</body></html>;
}
