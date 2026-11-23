import type { Metadata, Viewport } from "next";
import { Noto_Sans_Thai_Looped } from "next/font/google";
import "./globals.css";
import "./workspace-layout.css";
import "./mobile.css";
import "./typography.css";
import "../components/workspace-sidebar.css";
import "../components/date-input.css";
import "../components/reply-composer.css";
import "../components/quick-replies.css";
import "../components/chat-linked-content.css";
import "../components/image-preview.css";
import "../components/customer-notes.css";
import "../components/team.css";
import "../components/brand-combobox.css";
import "../components/shop-dashboard.css";
import "../components/shop-settings.css";
import "../components/shipping-workspace.css";

const notoSansThaiLooped = Noto_Sans_Thai_Looped({ subsets: ["thai", "latin"], weight: ["300", "400", "500", "600", "700"], variable: "--font-noto-sans-thai-looped", display: "swap" });
export const metadata: Metadata = { title: "Alex | หลังบ้านแอดมิน", description: "จัดการแชทลูกค้าและเตรียมร่างออเดอร์", robots: { index: false, follow: false } };
export const viewport: Viewport = { width: "device-width", initialScale: 1, viewportFit: "cover", interactiveWidget: "resizes-content" };
export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="th"><body className={notoSansThaiLooped.variable}>{children}</body></html>;
}
