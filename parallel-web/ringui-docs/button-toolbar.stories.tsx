import Button from '@jetbrains/ring-ui-built/components/button/button';
import ButtonGroup from '@jetbrains/ring-ui-built/components/button-group/button-group';
import ButtonToolbar from '@jetbrains/ring-ui-built/components/button-toolbar/button-toolbar';

export default {
  title: 'Components/Button Toolbar',

  parameters: {
    notes: 'Displays a toolbar with several buttons.',
  },
};

export const buttonToolbar = () => (
  <ButtonToolbar>
    <Button primary delayed>
      Run
    </Button>
    <ButtonGroup>
      <Button>Button one</Button>
      <Button>Button two</Button>
      <Button disabled>Button three</Button>
    </ButtonGroup>
    <Button>Another action</Button>
  </ButtonToolbar>
);
