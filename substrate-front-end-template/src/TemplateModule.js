import React, {useEffect, useState} from 'react'
import {Form, Input, Grid, Card, Statistic} from 'semantic-ui-react'

import {useSubstrateState} from './substrate-lib'
import {TxButton} from './substrate-lib/components'

//小端模式
//number 要转换的整形数值
//length 要转成什么byte数组，规定数组的长度
//如uint16，则lenght=2表示两个字节，转成的byte数组长度是length=2
//如uint32，则lenght=2表示两个字节，转成的byte数组长度是length=4
function IntToBytesLittleEndian(number, length) {
    var bytes = [];
    var i = 0;
    do {
        bytes[i++] = number & (255);
        number = number >> 8;
    } while (i < length)
    return bytes;
}


//小端模式
function BytesToIntLittleEndian(bytes) {
    var val = 0;
    for (var i = bytes.length - 1; i >= 0; i--) {
        val += bytes[i];
        if (i != 0) {
            val = val << 8;
        }
    }
    return val;
}

//字符串转字节序列
function stringToByte(str) {
    var bytes = new Array();
    var len, c;
    len = str.length;
    for (var i = 0; i < len; i++) {
        c = str.charCodeAt(i);
        if (c >= 0x010000 && c <= 0x10FFFF) {
            bytes.push(((c >> 18) & 0x07) | 0xF0);
            bytes.push(((c >> 12) & 0x3F) | 0x80);
            bytes.push(((c >> 6) & 0x3F) | 0x80);
            bytes.push((c & 0x3F) | 0x80);
        } else if (c >= 0x000800 && c <= 0x00FFFF) {
            bytes.push(((c >> 12) & 0x0F) | 0xE0);
            bytes.push(((c >> 6) & 0x3F) | 0x80);
            bytes.push((c & 0x3F) | 0x80);
        } else if (c >= 0x000080 && c <= 0x0007FF) {
            bytes.push(((c >> 6) & 0x1F) | 0xC0);
            bytes.push((c & 0x3F) | 0x80);
        } else {
            bytes.push(c & 0xFF);
        }
    }
    return bytes;

}

function byteToString(arr) {
    if (typeof arr === 'string') {
        return arr;
    }
    var str = '',
        _arr = arr;
    for (var i = 0; i < _arr.length; i++) {
        var one = _arr[i].toString(2),
            v = one.match(/^1+?(?=0)/);
        if (v && one.length == 8) {
            var bytesLength = v[0].length;
            var store = _arr[i].toString(2).slice(7 - bytesLength);
            for (var st = 1; st < bytesLength; st++) {
                store += _arr[st + i].toString(2).slice(2);
            }
            str += String.fromCharCode(parseInt(store, 2));
            i += bytesLength - 1;
        } else {
            str += String.fromCharCode(_arr[i]);
        }
    }
    return str;
}

function Main(props) {
    const {api} = useSubstrateState()

    // The transaction submission status
    const [status, setStatus] = useState('')

    // The currently stored value
    const [currentValue, setCurrentValue] = useState("None")
    const [formValue, setFormValue] = useState(0)

    useEffect(() => {
        let prefix = 'pallet_example::indexing::';
        let key = stringToByte(prefix).concat(IntToBytesLittleEndian(formValue, 4));
        console.log(key);
        api.rpc.offchain.localStorageGet('PERSISTENT', key).then((res) => {
            console.log('res = ', res)
            if (res.isNone) {
                setCurrentValue("None")
                return
            }
            let value = byteToString(res.value.slice(0, 10)) + BytesToIntLittleEndian(res.value.slice(10, res.value.length));
            setCurrentValue(value);
        })
    }, [api.rpc.offchain, formValue])

    return (<Grid.Column width={8}>
        <h1>Template Module</h1>
        <Card centered>
            <Card.Content textAlign="center">
                <Statistic label="Current Value" value={currentValue}/>
            </Card.Content>
        </Card>
        <Form>
            <Form.Field>
                <Input
                    label="New Value"
                    state="newValue"
                    type="number"
                    onChange={(_, {value}) => setFormValue(value)}
                />
            </Form.Field>
            <Form.Field style={{textAlign: 'center'}}>
                <TxButton
                    label="Store Something"
                    type="SIGNED-TX"
                    setStatus={setStatus}
                    attrs={{
                        palletRpc: 'templateModule',
                        callable: 'doSomething',
                        inputParams: [formValue],
                        paramFields: [true],
                    }}
                />
            </Form.Field>
            <div style={{overflowWrap: 'break-word'}}>{status}</div>
        </Form>
    </Grid.Column>)
}

export default function TemplateModule(props) {
    const {api} = useSubstrateState()
    return api.query.templateModule && api.query.templateModule.something ? (<Main {...props} />) : null
}
