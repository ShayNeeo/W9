import type { Metadata } from "next";
import "./globals.css";
import Header from "../components/Header";
import Footer from "../components/Footer";

export const metadata: Metadata = {
    title: "W9 Tools",
    description: "Fast drops, Short links, Secure notes",
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="en">
            <body>
                <div className="app">
                    <Header />
                    {children}
                    <Footer />
                </div>
            </body>
        </html>
    );
}
