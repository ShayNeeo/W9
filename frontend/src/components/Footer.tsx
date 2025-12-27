export default function Footer() {
    return (
        <footer className="site-footer">
            <div className="footer-columns">
                <div>
                    <div className="footer-title">Developed by W9 Labs</div>
                    <p className="footer-copy">
                        W9 Tools powers the public landing page for the W9 Labs network. Short links, note drops, and admin surfaces are open-source
                        and community audited. Contact <a href="mailto:hi@w9.se">hi@w9.se</a>.
                    </p>
                </div>
                <div>
                    <div className="footer-title">Network</div>
                    <ul className="footer-links">
                        <li>
                            <a href="https://w9.se" target="_blank" rel="noreferrer">
                                W9 SE · HQ & Docs
                            </a>
                        </li>
                        <li>
                            <a href="https://w9.nu" target="_blank" rel="noreferrer">
                                W9 Mail · Transactional rail
                            </a>
                        </li>
                        <li>
                            <a href="https://reminder.w9.nu" target="_blank" rel="noreferrer">
                                W9 Daily Reminders · Calendar digest
                            </a>
                        </li>
                    </ul>
                </div>
                <div>
                    <div className="footer-title">Legal</div>
                    <ul className="footer-links">
                        <li><a href="/terms">Terms of Service</a></li>
                        <li><a href="/privacy">Privacy Notice</a></li>
                    </ul>
                </div>
            </div>
            <div className="footer-bottom">© {new Date().getFullYear()} W9 Labs · Open infrastructure for independent teams.</div>
        </footer>
    )
}
