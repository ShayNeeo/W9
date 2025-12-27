'use client'

import { useRef, useEffect } from 'react'

const TURNSTILE_SITE_KEY: string = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY || ''

declare global {
    interface Window {
        turnstile?: {
            render: (element: string | HTMLElement, options: {
                sitekey: string
                callback?: (token: string) => void
                'error-callback'?: () => void
                'expired-callback'?: () => void
            }) => string
            reset: (widgetId?: string) => void
            remove: (widgetId?: string) => void
        }
    }
}

export default function TurnstileWidget({ onVerify, onError }: { onVerify: (token: string) => void; onError?: () => void }) {
    const widgetRef = useRef<HTMLDivElement>(null)
    const widgetIdRef = useRef<string | null>(null)
    const onVerifyRef = useRef(onVerify)
    const onErrorRef = useRef(onError)

    useEffect(() => {
        onVerifyRef.current = onVerify
        onErrorRef.current = onError
    }, [onVerify, onError])

    useEffect(() => {
        if (!TURNSTILE_SITE_KEY || !widgetRef.current) return

        const checkTurnstile = () => {
            if (window.turnstile && widgetRef.current && !widgetIdRef.current) {
                widgetIdRef.current = window.turnstile.render(widgetRef.current, {
                    sitekey: TURNSTILE_SITE_KEY,
                    callback: (token: string) => {
                        onVerifyRef.current(token)
                    },
                    'error-callback': () => {
                        if (onErrorRef.current) onErrorRef.current()
                    },
                    'expired-callback': () => {
                        if (widgetIdRef.current) {
                            window.turnstile?.reset(widgetIdRef.current)
                        }
                    }
                })
            } else if (!window.turnstile) {
                setTimeout(checkTurnstile, 100)
            }
        }

        checkTurnstile()

        return () => {
            if (widgetIdRef.current && window.turnstile) {
                window.turnstile.remove(widgetIdRef.current)
                widgetIdRef.current = null
            }
        }
    }, [])

    if (!TURNSTILE_SITE_KEY) {
        return null
    }

    return <div ref={widgetRef} style={{ marginTop: '1rem' }}></div>
}
