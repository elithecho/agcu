import type { Metadata } from 'next';
import { ThemeProvider } from '@/components/theme-provider';
import { roobert } from './fonts/roobert';
import { roobertMono } from './fonts/roobert-mono';
import './globals.css';

export const metadata: Metadata = {
  title: 'agcu — Agent Computer Use CLI for AI agents',
  description:
    'Control any desktop app from the terminal. Click buttons, type text, read screens — all through the accessibility tree. Built in Rust.',
  keywords: [
    'Computer use',
    'AI agent',
    'computer use',
    'accessibility',
    'CLI',
    'desktop automation',
    'Rust',
    'Electron',
    'CDP',
    'screen reader',
    'agcu',
  ],
  metadataBase: new URL('https://github.com/elithecho/agcu'),
  openGraph: {
    title: 'agcu — Agent Computer Use CLI for AI agents',
    description:
      'Control any desktop app from the terminal. Click buttons, type text, read screens. Built in Rust.',
    url: 'https://github.com/elithecho/agcu',
    siteName: 'agcu',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'agcu — Agent Computer Use CLI for AI agents',
    description:
      'Control any desktop app from the terminal. Click buttons, type text, read screens. Built in Rust.',
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${roobert.variable} ${roobertMono.variable} antialiased`}
      suppressHydrationWarning
    >
      <body className="font-sans">
        <ThemeProvider>{children}</ThemeProvider>
      </body>
    </html>
  );
}
