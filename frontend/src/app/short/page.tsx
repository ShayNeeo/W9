'use client'

import { useState, useRef, FormEvent, useEffect } from 'react'
import { API_BASE, joinUrl, toAbsoluteUrl, guessMimeFromName } from '@/lib/utils'

type SuccessResult = {
    success: true
    short_url: string
    qr_code_data: string | null
}

type ErrorResult = {
    success: false
    error: string
}

type Result = SuccessResult | ErrorResult

export default function ShortLinksPage() {
    const [urlInput, setUrlInput] = useState('')
    const [fileInput, setFileInput] = useState<File | null>(null)
    const [generateQr, setGenerateQr] = useState(false)
    const [customCode, setCustomCode] = useState('')
    const [isLoading, setIsLoading] = useState(false)
    const [result, setResult] = useState<SuccessResult | null>(null)
    const [error, setError] = useState<string | null>(null)
    const [dragOver, setDragOver] = useState(false)
    const [imagePreview, setImagePreview] = useState<string | null>(null)
    const [fileInfo, setFileInfo] = useState<{ name: string; type: string; sizeKB: number } | null>(null)
    const [copySuccess, setCopySuccess] = useState(false)

    const fileRef = useRef<HTMLInputElement | null>(null)

    useEffect(() => {
        return () => {
            if (imagePreview) {
                URL.revokeObjectURL(imagePreview)
            }
        }
    }, [imagePreview])

    function handleUrlChange(v: string) {
        setUrlInput(v)
        if (fileInput) {
            if (fileRef.current) fileRef.current.value = ''
            setFileInput(null)
        }
    }

    function handleFileChange(file: File | null) {
        if (imagePreview) {
            URL.revokeObjectURL(imagePreview)
        }

        setFileInput(file)
        if (file) {
            setUrlInput('')
            if (file.type.startsWith('image/')) {
                const url = URL.createObjectURL(file)
                setImagePreview(url)
                setFileInfo({ name: file.name, type: file.type || guessMimeFromName(file.name), sizeKB: Math.round(file.size / 1024 * 10) / 10 })
            } else {
                setImagePreview(null)
                setFileInfo({ name: file.name, type: file.type || guessMimeFromName(file.name), sizeKB: Math.round(file.size / 1024 * 10) / 10 })
            }
        } else {
            setImagePreview(null)
            setFileInfo(null)
        }
    }

    function handleCustomCodeChange(v: string) {
        const cleaned = v.replace(/[^a-zA-Z0-9-_]/g, '').toLowerCase()
        setCustomCode(cleaned)
    }

    function resetAll() {
        setUrlInput('')
        setFileInput(null)
        setImagePreview(null)
        setFileInfo(null)
        setCustomCode('')
        setGenerateQr(false)
        setResult(null)
        setError(null)
        if (fileRef.current) fileRef.current.value = ''
    }

    async function handleSubmit(e: FormEvent) {
        e.preventDefault()
        setIsLoading(true)
        setError(null)
        setResult(null)

        try {
            const hasUrl = urlInput.trim().length > 0
            const hasFile = !!fileInput
            if (!hasUrl && !hasFile) {
                setError('Please provide a URL or choose a file.')
                return
            }
            if (hasUrl && hasFile) {
                setError('Provide only one: URL or File, not both.')
                return
            }

            const form = new FormData()
            form.set('qr_required', generateQr ? 'true' : 'false')
            const trimmedCode = customCode.trim()
            if (trimmedCode) {
                form.set('custom_code', trimmedCode)
            }

            if (hasUrl) {
                form.set('content', urlInput.trim())
            } else if (hasFile && fileInput) {
                form.set('content', fileInput)
            }

            const token = localStorage.getItem('w9_token')
            const headers: HeadersInit = {}
            if (token) {
                headers['Authorization'] = `Bearer ${token}`
            }

            // Note: This calls the new w9-links-creator API
            const resp = await fetch(joinUrl(API_BASE, '/api/shorten'), {
                method: 'POST',
                headers,
                body: form,
            })

            const data = (await resp.json()) as Result
            if (!resp.ok || (data as ErrorResult).success === false) {
                const msg = (data as ErrorResult).error || `HTTP ${resp.status}`
                throw new Error(msg)
            }

            const ok = data as SuccessResult
            setResult(ok)
        } catch (err: any) {
            setError(err?.message || 'Unexpected error')
        } finally {
            setIsLoading(false)
        }
    }

    return (
        <main className="page">
            <section className="box">
                <h1>W9 Short Links</h1>
                <p className="subtitle">Drop a file or paste a URL · get a short code & QR in seconds.</p>

                <form onSubmit={handleSubmit} className="form">
                    <div
                        className={`dropzone ${dragOver ? 'dragover' : ''}`}
                        onDragOver={(e) => { e.preventDefault(); setDragOver(true) }}
                        onDragLeave={() => setDragOver(false)}
                        onDrop={(e) => {
                            e.preventDefault()
                            setDragOver(false)
                            const f = e.dataTransfer.files?.[0]
                            if (f) handleFileChange(f)
                        }}
                        onPaste={(e) => {
                            const items = e.clipboardData?.items
                            if (!items) return
                            for (let i = 0; i < items.length; i++) {
                                const it = items[i]
                                if (it.kind === 'file') {
                                    const f = it.getAsFile()
                                    if (f) { handleFileChange(f); break }
                                }
                                if (it.kind === 'string') {
                                    it.getAsString((text) => {
                                        if (/^https?:\/\//i.test(text.trim())) handleUrlChange(text.trim())
                                    })
                                }
                            }
                        }}
                    >
                        <div>Drop / paste files or URLs here (1 GiB max)</div>
                    </div>

                    <label className="label">
                        URL
                        <input
                            type="text"
                            placeholder="https://example.com"
                            value={urlInput}
                            onChange={(e) => handleUrlChange(e.target.value)}
                            className="input"
                        />
                    </label>

                    <label className="label">
                        File
                        <input
                            ref={fileRef}
                            type="file"
                            onChange={(e) => handleFileChange(e.target.files?.[0] || null)}
                            className="input"
                        />
                    </label>

                    <label className="label">
                        Custom short code (optional)
                        <div className="custom-code-row">
                            <span className="code-prefix">w9.se/s/</span>
                            <input
                                type="text"
                                value={customCode}
                                onChange={(e) => handleCustomCodeChange(e.target.value)}
                                className="input"
                                placeholder="my-link"
                                maxLength={32}
                            />
                        </div>
                        <span className="hint">Letters, numbers, '-' and '_'. Minimum 3 characters.</span>
                    </label>

                    {imagePreview && (
                        <div className="preview">
                            <img src={imagePreview} alt="preview" />
                        </div>
                    )}
                    {!imagePreview && fileInfo && (
                        <div className="status">
                            <div>Name: {fileInfo.name}</div>
                            <div>Type: {fileInfo.type || 'unknown'}</div>
                            <div>Size: {fileInfo.sizeKB} KB</div>
                        </div>
                    )}

                    <label className="checkbox" style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                        <input
                            type="checkbox"
                            checked={generateQr}
                            onChange={(e) => setGenerateQr(e.target.checked)}
                        />
                        Generate QR Code
                    </label>

                    <div className="actions">
                        <button type="submit" className="button" disabled={isLoading}>
                            {isLoading ? 'Submitting…' : 'Create'}
                        </button>
                        <button type="button" className="button ghost" onClick={resetAll}>
                            Reset
                        </button>
                    </div>
                </form>
            </section>

            {(isLoading || error || result) && (
                <section className="box">
                    <h2 className="section-title">Output</h2>
                    {isLoading && <div className="status">Submitting…</div>}
                    {error && <div className="status error">{error}</div>}
                    {result && (
                        <div className="status">
                            <div className="row" style={{ gap: '0.5rem', alignItems: 'center', display: 'flex', flexWrap: 'wrap' }}>
                                <span className="label-inline">Short URL: </span>
                                <a href={toAbsoluteUrl(result.short_url)} className="link" target="_blank" rel="noreferrer">
                                    {toAbsoluteUrl(result.short_url)}
                                </a>
                                <button
                                    onClick={async () => {
                                        try {
                                            await navigator.clipboard.writeText(toAbsoluteUrl(result.short_url))
                                            setCopySuccess(true)
                                            setTimeout(() => setCopySuccess(false), 2000)
                                        } catch (err) {
                                            console.error('Copy failed:', err)
                                        }
                                    }}
                                    className="button ghost"
                                >
                                    {copySuccess ? '✓ Copied' : 'Copy'}
                                </button>
                            </div>
                            {result.qr_code_data && (
                                <div className="qr" style={{ marginTop: '1rem' }}>
                                    <img src={result.qr_code_data} alt="QR code" />
                                </div>
                            )}
                        </div>
                    )}
                </section>
            )}
        </main>
    )
}
