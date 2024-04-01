import Button from "#components/form/Button.tsx";
import { api } from "#utils/api.ts";

export default function Signout() {
  const onClick = async () => {
    try {
      await api.get("/signout");
      window.location.href = "/signin";
    } catch (err) {
      console.error(err);
    }
  };
  return <Button onClick={onClick}>Sign out</Button>;
}
