import { Paper, Stack, Table, Text } from "@mantine/core";
import { formatVlanRanges } from "../utils/formatVlanRanges";

export type BridgeTableRow = {
  baseInterface: string;
  vlanTags: Array<number>;
};

type BridgeSummaryCardProps = {
  isConfigEmpty: boolean;
  showBridgeSummary: boolean;
  rows: ReadonlyArray<BridgeTableRow>;
  placeholderMessage: string;
};

export function BridgeSummaryCard({
  isConfigEmpty,
  showBridgeSummary,
  rows,
  placeholderMessage,
}: BridgeSummaryCardProps) {
  return (
    <Paper withBorder radius="md" p="lg">
      <Stack gap="sm">
        <div>
          <Text fw={600}>Bridge VLAN 情報</Text>
          <Text size="sm" c="dimmed">
            Lintの指摘がない場合のみ、base interfaceごとの割当を表示します。
          </Text>
        </div>
        {!isConfigEmpty && showBridgeSummary ? (
          <Table striped highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Base interface</Table.Th>
                <Table.Th>Bridge VLANs</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {rows.map((row) => (
                <Table.Tr key={row.baseInterface}>
                  <Table.Td>{row.baseInterface}</Table.Td>
                  <Table.Td>
                    <Text fw={500}>{formatVlanRanges(row.vlanTags)}</Text>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        ) : (
          <Text c="dimmed" size="sm">
            {placeholderMessage}
          </Text>
        )}
      </Stack>
    </Paper>
  );
}
