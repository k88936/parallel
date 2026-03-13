import Tag from '@jetbrains/ring-ui-built/components/tag/tag';
import Link from '@jetbrains/ring-ui-built/components/link/link';
import Group from '@jetbrains/ring-ui-built/components/group/group';

export default {
  title: 'Components/Group',

  parameters: {
    notes: 'Places inner components with fixed spacing between them.',
  },
};

export const basic = () => (
  <Group>
    <Tag>Tag</Tag>
    <span>Text</span>
    <Link>Link</Link>
  </Group>
);

basic.storyName = 'Group';
