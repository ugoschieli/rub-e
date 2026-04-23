import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { TeamSwitcher } from '../../components/team-switcher'
import React from 'react'
import { SidebarProvider } from '@/components/ui/sidebar'

describe('TeamSwitcher Component', () => {
  const mockTeams = [
    {
      name: 'Team 1',
      logo: '/logo1.png',
    },
    {
      name: 'Team 2',
      logo: '/logo2.png',
    },
  ]

  const renderWithSidebar = (ui: React.ReactElement) => {
    return render(
      <SidebarProvider>
        {ui}
      </SidebarProvider>
    )
  }

  it('renders the active team name', () => {
    renderWithSidebar(<div data-testid="wrapper"><TeamSwitcher teams={mockTeams} /></div>)
    
    expect(screen.getByText('Team 1')).toBeInTheDocument()
  })

  it('returns null if no teams are provided', () => {
    renderWithSidebar(<div data-testid="wrapper"><TeamSwitcher teams={[]} /></div>)
    const wrapper = screen.getByTestId('wrapper')
    expect(wrapper).toBeEmptyDOMElement()
  })
})
