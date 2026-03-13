import Link from '@jetbrains/ring-ui-built/components/link/link';
import Breadcrumbs from '@jetbrains/ring-ui-built/components/breadcrumbs/breadcrumbs';

export default {
  title: 'Components/Breadcrumbs',
  component: Breadcrumbs,
};

export const Default: Story = {
  render: args => (
    <Breadcrumbs {...args}>
      <Link href='/'>First Page</Link>
      <Link href='/'>Second Page</Link>
      <Link href='/'>Third Page</Link>
      <Link href='/' active>
        Current Page
      </Link>
    </Breadcrumbs>
  ),
};
