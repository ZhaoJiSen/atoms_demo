import type { Config } from 'tailwindcss'

export default <Partial<Config>>{
  theme: {
    extend: {
      colors: {
        canvas: '#010102',
        panel: '#0f1011',
        panel2: '#141516',
        panel3: '#18191a',
        line: '#23252a',
        lineStrong: '#34343a',
        ink: '#f7f8f8',
        muted: '#8a8f98',
        accent: '#5e6ad2',
      },
      fontFamily: {
        sans: [
          'Inter',
          'ui-sans-serif',
          'system-ui',
          '-apple-system',
          'BlinkMacSystemFont',
          'Segoe UI',
          'sans-serif',
        ],
      },
    },
  },
}
