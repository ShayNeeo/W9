import Link from 'next/link'

export default function Home() {
    return (
        <main className="page">
            <section className="box" style={{ textAlign: 'center' }}>
                <h1>W9 Tools</h1>
                <p className="subtitle" style={{ marginBottom: '1rem' }}>
                    <strong>Utilities for the Open Web</strong>
                </p>
                <p style={{ marginBottom: '1.5rem' }}>
                    This suite provides essential tools for sharing and collaboration without tracking or ads.
                    Shorten links, share files, and secure notes with ephemeral storage.
                </p>
                <div className="actions" style={{ justifyContent: 'center' }}>
                    <Link href="/short" className="button">Short Links</Link>
                    <Link href="/note" className="button">Notepad</Link>
                    <Link href="/convert" className="button ghost">Converter</Link>
                </div>
            </section>

            <section className="box">
                <h2 className="section-title">Features</h2>
                <ul className="list">
                    <li><strong>Short Links:</strong> Create short, memorable URLs for long links or files. Include QR codes instantly.</li>
                    <li><strong>Notepad:</strong> Share text snippets or markdown with optional password protection and burn-after-reading.</li>
                    <li><strong>Converter:</strong> (Coming Soon) Convert file formats locally in your browser.</li>
                    <li><strong>API Access:</strong> Integrate minimal tools into your own workflows.</li>
                </ul>
            </section>

            <section className="box warning">
                <h2 className="section-title">Privacy Notice</h2>
                <p>
                    Uploaded contents are public by default unless protected. Do not upload sensitive PII.
                    Files are retained for 30 days.
                </p>
            </section>
        </main>
    )
}
