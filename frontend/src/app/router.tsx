import { createBrowserRouter } from 'react-router'
import { HomePage } from '../features/home/HomePage'
import { Layout } from './Layout'

export const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [{ index: true, element: <HomePage /> }],
  },
])
