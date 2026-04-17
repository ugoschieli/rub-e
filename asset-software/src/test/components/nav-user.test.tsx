import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { NavUser } from '../../components/nav-user'
import React from 'react'
import { SidebarProvider } from '@/components/ui/sidebar'

// Mock UI components
vi.mock('@/components/ui/avatar', () => ({
  Avatar: ({ children }: any) => <div>{children}</div>,
  AvatarImage: ({ src, alt }: any) => <img src={src} alt={alt} />,
  AvatarFallback: ({ children }: any) => <div>{children}</div>,
}))

vi.mock('@/components/ui/dropdown-menu', () => ({
  DropdownMenu: ({ children }: any) => <div>{children}</div>,
  DropdownMenuTrigger: ({ children, asChild }: any) => <div data-testid="trigger">{children}</div>,
  DropdownMenuContent: ({ children }: any) => <div data-testid="content">{children}</div>,
  DropdownMenuItem: ({ children }: any) => <div>{children}</div>,
  DropdownMenuGroup: ({ children }: any) => <div>{children}</div>,
  DropdownMenuLabel: ({ children }: any) => <div>{children}</div>,
  DropdownMenuSeparator: () => <hr />,
}))

describe('NavUser Component', () => {
  const mockUser = {
    name: 'John Doe',
    email: 'john@example.com',
    avatar: '/avatar.png',
  }

  const renderWithSidebar = (ui: React.ReactElement) => {
    return render(
      <SidebarProvider>
        {ui}
      </SidebarProvider>
    )
  }

  it('renders user name and email', () => {
    renderWithSidebar(<NavUser user={mockUser} />)
    
    expect(screen.getAllByText(mockUser.name).length).toBeGreaterThanOrEqual(1)
    expect(screen.getAllByText(mockUser.email).length).toBeGreaterThanOrEqual(1)
  })

  it('renders avatar with correct src and alt', () => {
    renderWithSidebar(<NavUser user={mockUser} />)
    
    const avatarImages = screen.getAllByRole('img')
    const triggerAvatar = avatarImages.find(img => img.getAttribute('src') === mockUser.avatar)
    expect(triggerAvatar).toBeInTheDocument()
    expect(triggerAvatar).toHaveAttribute('alt', mockUser.name)
  })

  it('renders dropdown content', () => {
    renderWithSidebar(<NavUser user={mockUser} />)
    
    // Content is rendered directly in our mock
    expect(screen.getByText('Upgrade to Pro')).toBeInTheDocument()
    expect(screen.getByText('Account')).toBeInTheDocument()
    expect(screen.getByText('Billing')).toBeInTheDocument()
    expect(screen.getByText('Notifications')).toBeInTheDocument()
    expect(screen.getByText('Log out')).toBeInTheDocument()
  })
})
