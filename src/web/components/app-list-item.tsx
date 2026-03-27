import { Badge, Typography } from "@/components/ui";
import type { AppActivity } from "@/types/api";
import { productivityToVariant } from "@/types/api";
import { formatTime } from "@/utils/formatters";

interface AppListItemProps {
  app: AppActivity;
}

const AppListItem = ({ app }: AppListItemProps) => {
  const variant = productivityToVariant(app.productivity);

  return (
    <li className="flex items-center justify-between py-2 border-b border-gray-250 last:border-b-0">
      <span className="flex items-center gap-2">
        <Badge variant={variant} size="dot" />
        <Typography variant="body">{app.name}</Typography>
      </span>
      <Typography variant="time" color="secondary">
        {formatTime(app.duration_secs)}
      </Typography>
    </li>
  );
};

export type { AppListItemProps };
export { AppListItem };
