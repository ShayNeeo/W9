import type { Config } from "tailwindcss";

const config: Config = {
    content: [
        "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
        "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
        "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
    ],
    theme: {
        extend: {
            colors: {
                background: "var(--background)",
                foreground: "var(--foreground)",
                brand: {
                    dark: "#0b0c10",
                    gray: "#1f2833",
                    light: "#c5c6c7",
                    cyan: "#66fcf1",
                    teal: "#45a29e"
                }
            },
            fontFamily: {
                mono: ['"Courier New"', 'Courier', 'monospace'],
            },
        },
    },
    plugins: [],
};
export default config;
