import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { NavUser } from '../../components/nav-user'
import { TeamSwitcher } from '../../components/team-switcher'
import React from 'react'

// Mock Sidebar hook
vi.mock('../../components/ui/sidebar', () => ({
  useSidebar: () => ({ isMobile: false }),
  SidebarMenu: ({ children }: any) => <div>{children}</div>,
  SidebarMenuItem: ({ children }: any) => <div>{children}</div>,
  SidebarMenuButton: ({ children }: any) => <button>{children}</button>,
}))

// Mock UI components
vi.mock('../../components/ui/avatar', () => ({
  Avatar: ({ children }: any) => <div>{children}</div>,
  AvatarImage: ({ src, alt }: any) => <img src={src} alt={alt} />,
  AvatarFallback: ({ children }: any) => <span>{children}</span>,
}))

vi.mock('../../components/ui/dropdown-menu', () => ({
  DropdownMenu: ({ children }: any) => <div>{children}</div>,
  DropdownMenuTrigger: ({ children }: any) => <div>{children}</div>,
  DropdownMenuContent: ({ children }: any) => <div>{children}</div>,
  DropdownMenuItem: ({ children }: any) => <button>{children}</button>,
  DropdownMenuSeparator: () => <hr />,
  DropdownMenuLabel: ({ children }: any) => <div>{children}</div>,
  DropdownMenuGroup: ({ children }: any) => <div>{children}</div>,
}))

describe('NavUser', () => {
  const mockUser = {
    name: 'John Doe',
    email: 'john@example.com',
    avatar: 'avatar.png'
  }

  it('renders user details', () => {
    render(<NavUser user={mockUser} />)
    expect(screen.getAllByText('John Doe').length).toBeGreaterThan(0)
    expect(screen.getAllByText('john@example.com').length).toBeGreaterThan(0)
  })
})

describe('TeamSwitcher', () => {
  const mockTeams = [
    { name: 'Team A', logo: 'logo-a.png' },
    { name: 'Team B', logo: 'logo-b.png' }
  ]

  it('renders active team name', () => {
    render(<TeamSwitcher teams={mockTeams} />)
    expect(screen.getByText('Team A')).toBeInTheDocument()
  })
})
