import {useState} from 'react';
import {type StoryFn} from '@storybook/react-webpack5';

import Link from '@jetbrains/ring-ui-built/components/link/link';
import {type TableAttrs} from '@jetbrains/ring-ui-built/components/simple-table/table';
import SimpleTable from '@jetbrains/ring-ui-built/components/simple-table/simple-table';
import {type SelectionItem} from '@jetbrains/ring-ui-built/components/simple-table/selection';
import {type SortParams} from '@jetbrains/ring-ui-built/components/simple-table/header-cell';

import mock from '@jetbrains/ring-ui-built/components/simple-table/table.stories.json';
import tableData from '@jetbrains/ring-ui-built/components/simple-table/table.examples2.json';
/**
 * Simple stateless table without hover effect
 */
export default {
  title: 'Components/Simple Table',

  component: SimpleTable,
  parameters: {
    screenshots: {skip: true},
  },
  argTypes: {
    selection: {
      control: {disable: true},
    },
  },
};

interface Item extends SelectionItem {
  country: string;
  city: string;
  url: string;
  children?: Item[];
}
interface BasicDemoProps extends TableAttrs<Item> {
  withCaption: boolean;
}
const tdata = tableData.countries;
export const Basic: StoryFn<BasicDemoProps> = args => (
  <div>
    <SimpleTable {...args} data={tdata} />
  </div>
);
Basic.args = {
  columns: [
    {
      id: 'country',
      title: 'Country',
    },

    {
      id: 'id',
      title: 'ID',
      rightAlign: true,
    },

    {
      id: 'city',
      title: 'City',
      getDataTest: item => item.city,
    },

    {
      id: 'url',
      title: 'URL',
      getValue({url}) {
        return <Link href={url}>{url}</Link>;
      },
    },
  ],
  autofocus: true,
  isItemSelectable: item => item.id !== 14,
};
Basic.storyName = 'basic';

export const WithSorting: StoryFn<BasicDemoProps> = args => {
  const {onSort, onSelect, withCaption, onReorder, ...restProps} = args;
  const [sortKey, setSortKey] = useState<keyof Item>('country');
  const [sortOrder, setSortOrder] = useState<boolean>(true);
  const data = mock.toSorted((a, b) => String(a[sortKey]).localeCompare(String(b[sortKey])) * (sortOrder ? 1 : -1));

  const handleSort = (event: SortParams) => {
    onSort?.(event);
    setSortKey(event.column.id as keyof Item);
    setSortOrder(event.order);
  };

  return <SimpleTable {...restProps} data={data} onSort={handleSort} sortKey={sortKey} sortOrder={sortOrder} />;
};
WithSorting.args = {
  columns: [
    {
      id: 'country',
      title: 'Country',
      sortable: true,
    },

    {
      id: 'id',
      title: 'ID',
      rightAlign: true,
    },

    {
      id: 'city',
      title: 'City',
      getDataTest: item => item.city,
      sortable: true,
    },

    {
      id: 'url',
      title: 'URL',
      getValue({url}) {
        return <Link href={url}>{url}</Link>;
      },
    },
  ],
  autofocus: true,
  isItemSelectable: item => item.id !== 14,
};
WithSorting.argTypes = {
  data: {
    control: {disable: true},
  },
  sortKey: {
    control: {disable: true},
  },
  sortOrder: {
    control: {disable: true},
  },
  caption: {
    control: {disable: true},
  },
};
WithSorting.storyName = 'with sorting';
