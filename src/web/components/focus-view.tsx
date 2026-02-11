import { Card, CardBody, Typography } from "@/components/ui";
import { formatBudget } from "@/utils/formatters";

export type FocusViewProps = {
  budgetRemaining: number;
};

export const FocusView = ({ budgetRemaining }: FocusViewProps) => {
  return (
    <Card variant="active" className="mb-4">
      <CardBody className="py-6 text-center">
        <Typography variant="h2" color="productive" className="mb-4">
          Focus Mode Active
        </Typography>
        <div className="space-y-1">
          <Typography variant="label" color="muted">
            Budget remaining
          </Typography>
          <Typography variant="budget">
            {formatBudget(budgetRemaining)}
          </Typography>
        </div>
      </CardBody>
    </Card>
  );
};
