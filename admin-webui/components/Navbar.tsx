'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useEffect, useState } from 'react'

export default function Navbar() {
  const pathname = usePathname()
  const [bucketName, setBucketName] = useState<string>('')
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const token = localStorage.getItem('admin_token')
        if (!token) {
          setIsLoading(false)
          return
        }

        const res = await fetch('/api/config', {
          headers: {
            Authorization: `Bearer ${token}`
          }
        })

        if (res.ok) {
          const data = await res.json()
          setBucketName(data.bucket)
        }
      } catch (err) {
        console.error('Failed to fetch config:', err)
      } finally {
        setIsLoading(false)
      }
    }

    fetchConfig()
  }, [])

  const isActive = (path: string) => pathname === path

  return (
    <div className='sticky top-0 z-50 navbar bg-base-100/95 border-b border-base-300 shadow-sm backdrop-blur-sm'>
      <div className='flex-1'>
        <Link href='/' className='btn btn-ghost text-xl transition-transform duration-200 hover:scale-105'>
          Admin
        </Link>
        <div className="ml-4 text-sm opacity-60 shrink flex items-center min-w-[120px]">
          {isLoading ? (
            <div className="flex items-center gap-2 animate-pulse">
              <div className="h-3 w-16 bg-base-300 rounded"></div>
              <div className="h-3 w-24 bg-base-300 rounded"></div>
            </div>
          ) : bucketName ? (
            <span
              className="font-mono truncate block max-w-[200px] sm:max-w-full animate-in fade-in duration-200"
              title={bucketName}
            >
              Bucket: {bucketName}
            </span>
          ) : null}
        </div>
      </div>
      <div className='flex-none'>
        <ul className='menu menu-horizontal px-1'>
          <li>
            <Link
              href='/'
              className={`transition-all duration-200 ease-out ${isActive('/') ? 'active' : 'hover:bg-base-200'}`}
            >
              Uploader
            </Link>
          </li>
          <li>
            <Link
              href='/videos'
              className={`transition-all duration-200 ease-out ${isActive('/videos') ? 'active' : 'hover:bg-base-200'}`}
            >
              Videos
            </Link>
          </li>
        </ul>
      </div>
    </div>
  )
}
