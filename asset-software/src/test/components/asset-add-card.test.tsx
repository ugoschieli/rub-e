import { render, screen, fireEvent, waitFor, act } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { AssetAddCard } from '../../components/asset-add-card'
import * as services from '../../components/services'
import React from 'react'
import { useData } from '../../context/data-context'

// Mock context
vi.mock('../../context/data-context', () => ({
  useData: vi.fn()
}))

vi.mock('../../components/services', () => ({
  handleAddAsset: vi.fn(),
}))

// Mock UI components
vi.mock('../../components/ui/card', () => ({
  Card: ({ children }: any) => <div>{children}</div>,
  CardHeader: ({ children }: any) => <header>{children}</header>,
  CardTitle: ({ children }: any) => <h1>{children}</h1>,
  CardDescription: ({ children }: any) => <p>{children}</p>,
  CardContent: ({ children }: any) => <div>{children}</div>,
  CardFooter: ({ children }: any) => <footer>{children}</footer>,
}))

vi.mock('../../components/ui/select', () => ({
  Select: ({ children, onValueChange, value }: any) => (
    <div data-testid="select">
      <select 
        role="combobox" 
        value={value} 
        onChange={(e) => onValueChange(e.target.value)}
      >
        {children}
      </select>
    </div>
  ),
  SelectTrigger: ({ children }: any) => <div>{children}</div>,
  SelectValue: ({ placeholder }: any) => <span>{placeholder}</span>,
  SelectContent: ({ children }: any) => <>{children}</>,
  SelectItem: ({ value, children }: any) => <option value={value}>{children}</option>,
}))

vi.mock('../../components/ui/input', () => ({
  Input: (props: any) => <input {...props} />,
}))

vi.mock('../../components/ui/label', () => ({
  Label: ({ children, htmlFor }: any) => <label htmlFor={htmlFor}>{children}</label>,
}))

vi.mock('../../components/ui/button', () => ({
  Button: ({ children, onClick }: any) => <button onClick={onClick}>{children}</button>,
}))

describe('AssetAddCard', () => {
  const mockOnClose = vi.fn()
  const mockRefreshData = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(useData).mockReturnValue({
      projects: [{ id: 1, name: 'Proj 1' }],
      refreshData: mockRefreshData,
    } as any)
  })

  it('renders correctly', () => {
    render(<AssetAddCard onClose={mockOnClose} />)
    expect(screen.getByText('Created asset')).toBeInTheDocument()
    expect(screen.getByText('Proj 1')).toBeInTheDocument()
  })

  it('calls handleAddAsset on create', async () => {
    render(<AssetAddCard onClose={mockOnClose} />)
    
    fireEvent.change(screen.getByPlaceholderText('Asset name'), { target: { value: 'New Asset' } })
    
    const select = screen.getByRole('combobox')
    fireEvent.change(select, { target: { value: '1' } })
    
    await act(async () => {
      fireEvent.click(screen.getByText('Create Asset'))
    })
    
    expect(services.handleAddAsset).toHaveBeenCalledWith('New Asset', 1)
    expect(mockRefreshData).toHaveBeenCalled()
  })

  it('shows alert if fields are empty', () => {
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
    render(<AssetAddCard onClose={mockOnClose} />)
    
    fireEvent.click(screen.getByText('Create Asset'))
    
    expect(alertSpy).toHaveBeenCalledWith('Please fill all fields!')
    alertSpy.mockRestore()
  })

  it('calls onClose after timeout on success', async () => {
    vi.useFakeTimers()
    render(<AssetAddCard onClose={mockOnClose} />)
    
    fireEvent.change(screen.getByPlaceholderText('Asset name'), { target: { value: 'New Asset' } })
    fireEvent.change(screen.getByRole('combobox'), { target: { value: '1' } })
    
    await act(async () => {
      fireEvent.click(screen.getByText('Create Asset'))
    })
    
    expect(services.handleAddAsset).toHaveBeenCalled()
    
    await act(async () => {
        vi.advanceTimersByTime(2000)
    })
    
    expect(mockOnClose).toHaveBeenCalled()
    vi.useRealTimers()
  })
})
