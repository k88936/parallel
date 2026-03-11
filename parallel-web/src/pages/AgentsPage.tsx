import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Text from '@jetbrains/ring-ui-built/components/text/text';

export const AgentsPage = () => {
    return (
        <Island>
            <IslandHeader border>
                <Heading level={1}>Agents</Heading>
            </IslandHeader>
            <IslandContent>
                <Text>No agents connected</Text>
            </IslandContent>
        </Island>
    );
};
