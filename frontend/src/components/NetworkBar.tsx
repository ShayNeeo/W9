import Link from 'next/link'

const NETWORK_LINKS = [
    { id: 'tools', label: 'W9 Tools', description: 'tools.w9.nu · Links & drops', href: 'https://tools.w9.nu', external: false },
    { id: 'home', label: 'W9 Labs', description: 'w9.se · HQ & Docs', href: 'https://w9.se', external: true },
    { id: 'mail', label: 'W9 Mail', description: 'w9.nu · Transactional rail', href: 'https://w9.nu', external: true },
    { id: 'reminders', label: 'W9 Daily Reminders', description: 'reminder.w9.nu · Calendar digest', href: 'https://reminder.w9.nu', external: true },
] as const

export default function NetworkBar({ active }: { active: string }) {
    return (
        <div className="network-bar">
            <div>
                <span className="network-label">W9 Labs Network</span>
                <span className="network-tagline">Open-source automation for independent teams</span>
            </div>
            <nav className="network-links">
                {NETWORK_LINKS.map((link) => {
                    const isActive = active === link.id
                    const className = `network-link ${isActive ? 'active' : ''}`
                    return (
                        <a
                            key={link.id}
                            href={link.href}
                            target={link.external ? "_blank" : undefined}
                            rel={link.external ? "noreferrer" : undefined}
                            className={className}
                        >
                            <span>{link.label}</span>
                            <small>{link.description}</small>
                        </a>
                    )
                })}
            </nav>
        </div>
    )
}
