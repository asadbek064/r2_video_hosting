import type { Metadata } from 'next'
import { Geist, Geist_Mono } from 'next/font/google'
import AuthWrapper from '@/components/AuthWrapper'
import { UploadProvider } from '@/context/UploadContext'
import './globals.css'

const geistSans = Geist({
  variable: '--font-geist-sans',
  subsets: ['latin']
})

const geistMono = Geist_Mono({
  variable: '--font-geist-mono',
  subsets: ['latin']
})

export const metadata: Metadata = {
  title: 'Admin WebUI',
  description: 'Admin interface for r2_video_hosting video uploader and manager'
}

export default function RootLayout({
  children
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html data-theme="silk" lang='en'>
      <body className={`${geistSans.variable} ${geistMono.variable} antialiased`}>
        <AuthWrapper>
          <UploadProvider>{children}</UploadProvider>
        </AuthWrapper>
      </body>
    </html>
  )
}
