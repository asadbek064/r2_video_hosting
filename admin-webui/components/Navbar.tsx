'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useEffect, useState } from 'react'

export default function Navbar() {
  const pathname = usePathname()
  const [bucketName, setBucketName] = useState<string>('')

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const token = localStorage.getItem('admin_token')
        if (!token) return

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
      }
    }

    fetchConfig()
  }, [])

  const isActive = (path: string) => pathname === path

  return (
    <div className='navbar bg-base-100 mb-8 border-b border-base-300'>
      <div className='flex-1'>
        <Link href='/' className='btn btn-ghost text-xl'>
          Admin
        </Link>
        {bucketName && (
          <div className='ml-4 text-sm opacity-60'>
            <span className='font-mono'>Bucket: {bucketName}</span>
          </div>
        )}
      </div>
      <div className='flex-none'>
        <ul className='menu menu-horizontal px-1'>
          <li>
            <Link href='/' className={isActive('/') ? 'active' : ''}>
              Uploader
            </Link>
          </li>
          <li>
            <Link href='/videos' className={isActive('/videos') ? 'active' : ''}>
              Videos
            </Link>
          </li>
          <li>
            <Link href='/analytics' className={isActive('/analytics') ? 'active' : ''}>
              Analytics
            </Link>
          </li>
        </ul>
      </div>
    </div>
  )
}
