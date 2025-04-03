/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        files: ["*.html", "./src/**/*.rs"],
        transform: {
            rs: (content) => content.replace(/(?:^|\s)class:/g, ' '),
        },
    },
    darkMode: 'class',
    theme: {
        extend: {
            colors: {
                brand: '#000000',
                accent1: '#4f4f4f',
                accent2: '#b3b3b3',
                highlight: '#e0e0e0',
                background: {
                    primary: '#f5f5f5',
                    secondary: '#e0e0e0',
                },
                darkBackground: '#1a1a1a',
                darkAccent: '#4f4f4f',
            },
            gradientColorStops: {
                primary: ['#4f4f4f', '#b3b3b3'],
            },
        },
    },
    plugins: [],
}
