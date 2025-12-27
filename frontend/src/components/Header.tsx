'use client'

import NetworkBar from './NetworkBar'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useEffect, useState } from 'react'

export default function Header() {
    const pathname = usePathname()
    const [token, setToken] = useState<string | null>(null)

    useEffect(() => {
        setToken(localStorage.getItem('w9_token'))

        const checkToken = () => {
            setToken(localStorage.getItem('w9_token'))
        }
        window.addEventListener('storage', checkToken)
        return () => window.removeEventListener('storage', checkToken)
    }, [])

    const handleLogout = () => {
        localStorage.removeItem('w9_token')
        setToken(null)
        window.location.href = '/'
    }

    const isHomepage = pathname === '/'

    return (
        <header className="header">
            <NetworkBar active="tools" />
            {!isHomepage && (
                <div className="brand">
                    <div>
                        <p className="eyebrow">Developed by W9 Labs</p>
                        <h1>W9 Tools</h1>
                        <span>Fast drops • Short links • Secure notes</span>
                    </div>
                    <div className="pill" style={{ borderColor: token ? '#00ffd0' : undefined, color: token ? '#00ffd0' : undefined }}>
                        {token ? 'SIGNED IN' : 'GUEST'}
                    </div>
                </div>
            )}
            <nav className="nav">
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem', alignItems: 'center' }}>
                    <Link href="/" className={pathname === '/' ? 'nav-link active' : 'nav-link'}>Home</Link>
                    <Link href="/short" className={pathname.startsWith('/short') ? 'nav-link active' : 'nav-link'}>Short Links</Link>
                    <Link href="/note" className={pathname.startsWith('/note') ? 'nav-link active' : 'nav-link'}>Notepad</Link>
                    <Link href="/convert" className={pathname.startsWith('/convert') ? 'nav-link active' : 'nav-link'}>Converter</Link>
                    <Link href="/profile" className={pathname === '/profile' ? 'nav-link active' : 'nav-link'}>Profile</Link>
                    <Link href="/admin" className={pathname.startsWith('/admin') ? 'nav-link active' : 'nav-link'}>Admin</Link>
                    <Link href="/terms" className={pathname === '/terms' ? 'nav-link active' : 'nav-link'}>Terms</Link>
                    <Link href="/privacy" className={pathname === '/privacy' ? 'nav-link active' : 'nav-link'}>Privacy</Link>
                </div>
                <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                    {token ? (
                        <button onClick={handleLogout} className="button secondary">Logout</button>
                    ) : (
                        <>
                            <Link href="/login" className={pathname === '/login' ? 'nav-link active' : 'nav-link'}>Login</Link>
                            <Link href="/register" className={pathname === '/register' ? 'nav-link active' : 'nav-link'}>Register</Link>
                        </>
                    )}
                </div>
            </nav>
        </header>
    )
}
