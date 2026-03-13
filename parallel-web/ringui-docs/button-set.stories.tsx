import Button from '@jetbrains/ring-ui-built/components/button/button';
import ButtonSet from '@jetbrains/ring-ui-built/components/button-set/button-set';

export default {
  title: 'Components/Button Set',

  parameters: {
    notes: 'Allows to group several buttons and ensures that margins between them are consistent.',
  },
};

export const buttonSet = () => (
  <ButtonSet>
    <Button>1st button</Button>
    <Button>2nd button</Button>
    <Button>3rd button</Button>
  </ButtonSet>
);
