import React, { useEffect, useState } from "react";

import { Form, Grid } from "semantic-ui-react";

import KittyCards from "./KittyCards";
import { TxButton } from "./substrate-lib/components";
import { useSubstrateState } from "./substrate-lib";
import { hexToU8a } from "@polkadot/util";

export default function Main(props) {
  const { api, currentAccount } = useSubstrateState();
  const [kittyCnt, setKittyCnt] = useState(0);
  const [status, setStatus] = useState("");
  const [kitties, setKitties] = useState([]);

  // 小猫的总数量
  useEffect(() => {
    let unsubscribe;
    api.query.kittiesModule
      .nextKittyId((newValue) => {
        setKittyCnt(newValue.toNumber());
      })
      .then((unsub) => {
        unsubscribe = unsub;
      })
      .catch(console.error);
    return () => unsubscribe && unsubscribe();
  }, [api.query.kittiesModule]);

  // // 小猫的数组
  useEffect(() => {
    let unsubscribe;
    api.query.kittiesModule.webKitties
      .entries((kittys) => {
        let newKitties = [];
        kittys.forEach(([key, kitty]) => {
          let val = kitty.unwrap().toHuman();
          newKitties.push({
            id: val.id,
            dna: hexToU8a(val.dna),
            owner: val.owner,
          });
        });
        setKitties(newKitties);
      })
      .then((unsub) => {
        unsubscribe = unsub;
      })
      .catch(console.error);
    return () => unsubscribe && unsubscribe();
  }, [api.query.kittiesModule, kittyCnt]); // kittyCnt 总数量更新,数据一并更新

  return (
    <Grid.Column width={16}>
      <h1>小毛孩</h1>
      <KittyCards kitties={kitties} accountPair={currentAccount} setStatus={setStatus}></KittyCards>
      <Form style={{ margin: "1em 0" }}>
        <Form.Field style={{ textAlign: "center" }}>
          <TxButton
            label={"创建小毛孩. " + kittyCnt}
            setStatus={setStatus}
            type="SIGNED-TX"
            attrs={{
              palletRpc: "kittiesModule",
              callable: "create",
              inputParams: [],
              paramFields: [],
            }}
          />
        </Form.Field>
      </Form>
      <div style={{ overflowWrap: "break-word" }}>status: {status}</div>
    </Grid.Column>
  );
}
