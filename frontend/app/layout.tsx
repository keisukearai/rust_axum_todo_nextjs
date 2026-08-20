import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "TODO",
  description: "Rust (Axum) + Next.js + PostgreSQL の TODO アプリ",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="ja">
      <body>{children}</body>
    </html>
  );
}
