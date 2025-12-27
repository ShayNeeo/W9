export const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || ''

export const joinUrl = (base: string, path: string) => {
    if (!base) return path
    if (base.endsWith('/')) base = base.slice(0, -1)
    if (!path.startsWith('/')) path = `/${path}`
    return `${base}${path}`
}

export const toAbsoluteUrl = (url: string) => {
    if (!url) return ''
    if (url.startsWith('http')) return url
    if (url.startsWith('/')) return `${window.location.origin}${url}`
    return `${window.location.origin}/${url}`
}

export const guessMimeFromName = (name: string) => {
    const ext = name.split('.').pop()?.toLowerCase()
    if (!ext) return 'application/octet-stream'
    const map: Record<string, string> = {
        png: 'image/png',
        jpg: 'image/jpeg',
        jpeg: 'image/jpeg',
        gif: 'image/gif',
        webp: 'image/webp',
        svg: 'image/svg+xml',
        txt: 'text/plain',
        pdf: 'application/pdf',
        zip: 'application/zip',
        json: 'application/json',
        md: 'text/markdown'
    }
    return map[ext] || 'application/octet-stream'
}
